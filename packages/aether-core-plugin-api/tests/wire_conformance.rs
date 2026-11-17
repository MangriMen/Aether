//! The published `plugin-wire` schema and the bytes the codec writes must describe the same
//! values. For each type that crosses the boundary, values are made from the schema itself
//! (every variant of every enum at least once), read as the Rust type, written with
//! `wire::encode`, read back as generic msgpack and validated against the schema. A field renamed,
//! a tag changed or a required field dropped on one side only fails here.

use aether_core_plugin_api::v0::{self, wire};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

const SCHEMA: &str = include_str!("../src/v0/schemas/plugin-wire.schema.json");

/// Deeper than this, a recursive type is cut at its simplest value.
const MAX_DEPTH: usize = 5;

struct Gen<'a> {
    defs: &'a Map<String, Value>,
}

impl Gen<'_> {
    /// Values that between them use every variant the schema has, each once at least.
    fn samples(&self, schema: &Value, depth: usize) -> Vec<Value> {
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let name = reference.strip_prefix("#/$defs/").expect("local ref");
            return self.samples(&self.defs[name], depth);
        }
        if let Some(constant) = schema.get("const") {
            return vec![constant.clone()];
        }
        if let Some(values) = schema.get("enum").and_then(Value::as_array) {
            return values.clone();
        }
        for key in ["oneOf", "anyOf"] {
            if let Some(branches) = schema.get(key).and_then(Value::as_array) {
                let branches: Vec<_> = if depth >= MAX_DEPTH {
                    branches.iter().take(1).collect()
                } else {
                    branches.iter().collect()
                };
                return branches
                    .into_iter()
                    .flat_map(|branch| self.samples(branch, depth + 1))
                    .collect();
            }
        }

        let kind = match schema.get("type") {
            Some(Value::Array(kinds)) => kinds
                .iter()
                .filter_map(Value::as_str)
                .find(|k| *k != "null"),
            Some(kind) => kind.as_str(),
            None => None,
        };
        match kind {
            Some("object") => self.object(schema, depth),
            Some("array") => {
                if depth >= MAX_DEPTH {
                    return vec![json!([])];
                }
                let item = schema.get("items").expect("array items");
                self.samples(item, depth + 1)
                    .into_iter()
                    .map(|sample| json!([sample]))
                    .collect()
            }
            Some("string") => vec![json!("x")],
            Some("integer") => vec![json!(
                schema.get("minimum").and_then(Value::as_i64).unwrap_or(0)
            )],
            Some("number") => vec![json!(0.5)],
            Some("boolean") => vec![json!(false), json!(true)],
            Some("null") => vec![Value::Null],
            // `{}`: any value.
            _ => vec![json!("any")],
        }
    }

    fn object(&self, schema: &Value, depth: usize) -> Vec<Value> {
        // A map: a key and its value.
        if let Some(values) = schema.get("additionalProperties").filter(|v| v.is_object()) {
            return self
                .samples(values, depth + 1)
                .into_iter()
                .map(|sample| json!({ "k": sample }))
                .collect();
        }

        let properties = schema
            .get("properties")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let required: Vec<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|names| names.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();

        let per_property: Vec<(&String, Vec<Value>)> = properties
            .iter()
            .map(|(name, property)| (name, self.samples(property, depth + 1)))
            .collect();

        let object_with = |pick: &dyn Fn(usize, &Vec<Value>) -> Option<Value>| {
            let mut object = Map::new();
            for (index, (name, options)) in per_property.iter().enumerate() {
                if let Some(value) = pick(index, options) {
                    object.insert((*name).clone(), value);
                }
            }
            Value::Object(object)
        };

        // The first value of everything, then one more object per further value of a property.
        let mut all = vec![object_with(&|_, options| options.first().cloned())];
        for (varying, (_, options)) in per_property.iter().enumerate() {
            for extra in options.iter().skip(1) {
                all.push(object_with(&|index, options| {
                    if index == varying {
                        Some(extra.clone())
                    } else {
                        options.first().cloned()
                    }
                }));
            }
        }
        // Optional properties are also left out once, which is what a plugin that does not know
        // them sends.
        if per_property
            .iter()
            .any(|(name, _)| !required.contains(&name.as_str()))
        {
            all.push(object_with(&|index, options| {
                let name = per_property[index].0;
                required
                    .contains(&name.as_str())
                    .then(|| options.first().cloned())
                    .flatten()
            }));
        }
        all
    }
}

/// msgpack as JSON for the validator: `bin` has no JSON form and becomes a string.
fn to_json(value: &rmpv::Value) -> Value {
    match value {
        rmpv::Value::Nil => Value::Null,
        rmpv::Value::Boolean(b) => json!(b),
        rmpv::Value::Integer(i) => i
            .as_i64()
            .map(Value::from)
            .or_else(|| i.as_u64().map(Value::from))
            .expect("integer"),
        rmpv::Value::F32(f) => json!(f),
        rmpv::Value::F64(f) => json!(f),
        rmpv::Value::String(s) => json!(s.as_str().expect("utf-8")),
        rmpv::Value::Binary(_) => json!("<bin>"),
        rmpv::Value::Array(items) => Value::Array(items.iter().map(to_json).collect()),
        rmpv::Value::Map(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.as_str().expect("string key").to_owned(), to_json(value)))
                .collect(),
        ),
        rmpv::Value::Ext(..) => panic!("an ext type is not part of the contract"),
    }
}

/// Whether a msgpack value holds a `bin` where `schema` says `x-msgpack: bin`, checked on the
/// value as msgpack since the JSON form lost it.
fn bins_are_bins(value: &rmpv::Value, schema: &Value, defs: &Map<String, Value>) {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let name = reference.strip_prefix("#/$defs/").unwrap();
        return bins_are_bins(value, &defs[name], defs);
    }
    if schema.get("x-msgpack").and_then(Value::as_str) == Some("bin") {
        assert!(value.is_bin(), "{value:?} is not a bin");
        return;
    }
    match (value, schema.get("properties").and_then(Value::as_object)) {
        (rmpv::Value::Map(entries), Some(properties)) => {
            for (key, value) in entries {
                if let Some(property) = properties.get(key.as_str().unwrap()) {
                    bins_are_bins(value, property, defs);
                }
            }
        }
        (rmpv::Value::Array(items), _) => {
            if let Some(item_schema) = schema.get("items") {
                for item in items {
                    bins_are_bins(item, item_schema, defs);
                }
            }
        }
        _ => {}
    }
}

/// Every field the schema names is `camelCase`. The codec and the schema agree on a `snake_case` key
/// just as well as on a camelCase one, so only this tells a slip from a decision.
#[test]
fn every_field_name_in_the_schema_is_camel_case() {
    fn walk(schema: &Value, path: &str, bad: &mut Vec<String>) {
        match schema {
            Value::Object(object) => {
                if let Some(Value::Object(properties)) = object.get("properties") {
                    for name in properties.keys() {
                        if name.contains('_') || name.chars().next().is_some_and(char::is_uppercase)
                        {
                            bad.push(format!("{path}/{name}"));
                        }
                    }
                }
                for (key, value) in object {
                    walk(value, &format!("{path}/{key}"), bad);
                }
            }
            Value::Array(items) => {
                for (index, value) in items.iter().enumerate() {
                    walk(value, &format!("{path}/{index}"), bad);
                }
            }
            _ => {}
        }
    }

    let bundle: Value = serde_json::from_str(SCHEMA).expect("schema");
    let mut bad = Vec::new();
    walk(&bundle, "", &mut bad);

    assert!(bad.is_empty(), "not camelCase: {bad:?}");
}

/// Every value of an enum and every tag is `camelCase`, and an enum with data is tagged `type`
/// (`status` for a result). A `snake_case` value or a `kind` tag passes the codec just as well, so
/// only this tells a slip from a decision.
#[test]
fn every_enum_value_and_tag_in_the_schema_is_camel_case_and_tagged_type() {
    fn is_camel_case(value: &str) -> bool {
        value.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && value.chars().all(|c| c.is_ascii_alphanumeric())
    }

    fn walk(schema: &Value, path: &str, bad: &mut Vec<String>) {
        match schema {
            Value::Object(object) => {
                let strings = object.get("const").into_iter().chain(
                    object
                        .get("enum")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten(),
                );
                for value in strings.filter_map(Value::as_str) {
                    if !is_camel_case(value) {
                        bad.push(format!("{path}: value `{value}`"));
                    }
                }
                if let Some(Value::Array(branches)) = object.get("oneOf") {
                    for (index, branch) in branches.iter().enumerate() {
                        let unit = branch.get("type").and_then(Value::as_str) == Some("string")
                            && branch.get("const").is_some();
                        let externally_tagged = branch
                            .get("properties")
                            .and_then(Value::as_object)
                            .is_some_and(|properties| {
                                properties.len() == 1
                                    && properties.values().all(|p| p.get("const").is_none())
                                    && branch.get("additionalProperties")
                                        == Some(&Value::Bool(false))
                            });
                        let mixed = branches
                            .iter()
                            .any(|b| b.get("type").and_then(Value::as_str) == Some("object"));
                        if (unit && mixed) || externally_tagged {
                            bad.push(format!(
                                "{path}/oneOf/{index}: a variant not written as {{type, data}}"
                            ));
                        }
                    }
                }
                if let Some(Value::Object(properties)) = object.get("properties") {
                    let tag = properties
                        .iter()
                        .find(|(_, property)| property.get("const").is_some());
                    if let Some((key, _)) = tag
                        && !["type", "status"].contains(&key.as_str())
                    {
                        bad.push(format!("{path}: tag `{key}`"));
                    }
                }
                for (key, value) in object {
                    walk(value, &format!("{path}/{key}"), bad);
                }
            }
            Value::Array(items) => {
                for (index, value) in items.iter().enumerate() {
                    walk(value, &format!("{path}/{index}"), bad);
                }
            }
            _ => {}
        }
    }

    let bundle: Value = serde_json::from_str(SCHEMA).expect("schema");
    let mut bad = Vec::new();
    walk(&bundle, "", &mut bad);

    assert!(
        bad.is_empty(),
        "not camelCase or not tagged `type`: {bad:#?}"
    );
}

/// A copy of the schema that refuses a key it does not list. The published schema leaves them
/// open so that a plugin may be newer than the host, but what the codec *writes* must be listed,
/// or a renamed field would pass unnoticed.
fn strictify(schema: &mut Value) {
    match schema {
        Value::Object(object) => {
            if object.contains_key("properties") {
                object.insert("additionalProperties".to_owned(), json!(false));
            }
            object.values_mut().for_each(strictify);
        }
        Value::Array(items) => items.iter_mut().for_each(strictify),
        _ => {}
    }
}

fn check<T: Serialize + DeserializeOwned>(property: &str) {
    let bundle: Value = serde_json::from_str(SCHEMA).expect("schema");
    let defs = bundle["$defs"].as_object().expect("$defs");
    let schema = &bundle["properties"][property];
    assert!(!schema.is_null(), "no property `{property}` in the schema");

    let mut strict = json!({
        "$defs": defs,
        "allOf": [schema],
    });
    strictify(&mut strict);
    let validator = jsonschema::validator_for(&strict).expect("schema compiles");

    let samples = Gen { defs }.samples(schema, 0);
    assert!(!samples.is_empty());

    for sample in samples {
        let value: T = serde_json::from_value(sample.clone())
            .unwrap_or_else(|error| panic!("`{property}`: {error} for {sample}"));
        let bytes = wire::encode(&value).expect("encodes");
        let msgpack = rmpv::decode::read_value(&mut bytes.as_slice()).expect("msgpack");
        let as_json = to_json(&msgpack);

        let errors: Vec<_> = validator
            .iter_errors(&as_json)
            .map(|e| e.to_string())
            .collect();
        assert!(
            errors.is_empty(),
            "`{property}`: {as_json} does not follow the schema: {errors:?}"
        );
        bins_are_bins(&msgpack, schema, defs);
    }
}

macro_rules! conforms {
    ($($name:ident: $property:literal => $ty:ty),+ $(,)?) => {
        $(#[test]
        fn $name() {
            check::<$ty>($property);
        })+

        /// Every property of the schema is checked above.
        #[test]
        fn every_property_of_the_schema_is_checked() {
            let bundle: Value = serde_json::from_str(SCHEMA).expect("schema");
            let mut checked = vec![$($property),+];
            checked.sort_unstable();
            let mut all: Vec<_> = bundle["properties"]
                .as_object()
                .expect("properties")
                .keys()
                .map(String::as_str)
                .collect();
            all.sort_unstable();
            assert_eq!(all, checked);
        }
    };
}

conforms! {
    host_error: "hostError" => v0::HostError,
    log: "log" => v0::LogParamsDto,
    java_version: "javaVersion" => v0::JavaVersionParamsDto,
    instance_id: "instanceId" => v0::InstanceIdParamsDto,
    change_contents: "changeContents" => v0::ChangeContentsParamsDto,
    command: "command" => v0::CommandDto,
    new_instance: "newInstance" => v0::NewInstanceDto,
    write_instance_file: "writeInstanceFile" => v0::WriteInstanceFileParamsDto,
    read_instance_file: "readInstanceFile" => v0::ReadInstanceFileParamsDto,
    http_request: "httpRequest" => v0::HttpRequestDto,
    http_post_json_request: "httpPostJsonRequest" => v0::HttpPostJsonRequestDto,
    download_to_cache: "downloadToCache" => v0::DownloadToCacheParamsDto,
    output: "output" => v0::OutputDto,
    java: "java" => v0::JavaDto,
    instance_dir: "instanceDir" => v0::InstanceDirDto,
    instance_created: "instanceCreated" => v0::InstanceCreatedDto,
    list_content_result: "listContentResult" => v0::ListContentResultDto,
    instance_file_bytes: "instanceFileBytes" => v0::InstanceFileBytesDto,
    http_response: "httpResponse" => v0::HttpResponseDto,
    cached_file: "cachedFile" => v0::CachedFileDto,
    internal_event: "internalEvent" => v0::PluginInternalEventDto,
    import_instance: "importInstance" => v0::PluginImportInstanceDto,
    update_instance_params: "updateInstanceParams" => v0::UpdateInstanceParamsDto,
    get_content_params: "getContentParams" => v0::GetContentParamsDto,
    list_versions_params: "listVersionsParams" => v0::ListVersionsParamsDto,
    get_version_params: "getVersionParams" => v0::GetVersionParamsDto,
    content_search_params: "contentSearchParams" => v0::ContentSearchParamsDto,
    resolve_file_params: "resolveFileParams" => v0::ResolveFileParamsDto,
    resolve_install_plan_params: "resolveInstallPlanParams" => v0::ResolveInstallPlanParamsDto,
    pack_source: "packSource" => v0::PackSourceDto,
    get_contents_metadata_params: "getContentsMetadataParams" => v0::GetContentsMetadataParamsDto,
    get_versions_metadata_params: "getVersionsMetadataParams" => v0::GetVersionsMetadataParamsDto,
    get_authors_params: "getAuthorsParams" => v0::GetAuthorsParamsDto,
    check_compatibility_params: "checkCompatibilityParams" => v0::PluginCheckCompatibilityParamsDto,
    content_search_result: "contentSearchResult" => v0::ContentSearchResultDto,
    content_item: "contentItem" => v0::ContentItemDto,
    content_version: "contentVersion" => v0::ContentVersionDto,
    list_versions_result: "listVersionsResult" => v0::ListVersionsResultDto,
    resolved_file: "resolvedFile" => v0::ResolvedFileDto,
    install_plan: "installPlan" => v0::InstallPlanDto,
    pack_metadata: "packMetadata" => v0::PackMetadataDto,
    get_contents_metadata_result: "getContentsMetadataResult" => v0::GetContentsMetadataResultDto,
    get_versions_metadata_result: "getVersionsMetadataResult" => v0::GetVersionsMetadataResultDto,
    get_authors_result: "getAuthorsResult" => v0::GetAuthorsResultDto,
    check_compatibility_result: "checkCompatibilityResult" => v0::CheckCompatibilityResultDto,
}
