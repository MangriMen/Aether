use std::str::FromStr;

use aether_core_plugin_api::v0::{
    ApiConfigDto, LoadConfigDto, PathMappingDto, PluginManifestDto, PluginMetadataDto,
    RuntimeConfigDto,
};

use crate::features::plugins::domain::{
    ApiConfig, LoadConfig, LoadConfigType, ManifestError, PathMapping, PluginManifest,
    PluginMetadata, RuntimeConfig,
};

// ── DTO → Domain (fallible — semver parsing, id scheme) ──

impl TryFrom<PluginManifestDto> for PluginManifest {
    type Error = ManifestError;

    fn try_from(dto: PluginManifestDto) -> Result<Self, Self::Error> {
        Ok(Self {
            metadata: dto.metadata.try_into()?,
            runtime: dto.runtime.into(),
            load: dto.load.try_into()?,
            api: dto.api.try_into()?,
        })
    }
}

impl TryFrom<PluginMetadataDto> for PluginMetadata {
    type Error = ManifestError;

    fn try_from(dto: PluginMetadataDto) -> Result<Self, Self::Error> {
        let version =
            semver::Version::parse(&dto.version).map_err(|e| ManifestError::InvalidSemver {
                field: "metadata.version".to_owned(),
                value: dto.version.clone(),
                error: e.to_string(),
            })?;

        let metadata = Self {
            id: dto.id,
            name: dto.name,
            version,
            description: dto.description,
            authors: dto.authors,
            license: dto.license,
        };

        // Every manifest the core reads passes through here — on installation before any file
        // is copied, and on loading — so a plugin with an invalid or reserved id never gets in.
        metadata.validate_id()?;

        Ok(metadata)
    }
}

impl From<RuntimeConfigDto> for RuntimeConfig {
    fn from(dto: RuntimeConfigDto) -> Self {
        Self {
            allowed_hosts: dto.allowed_hosts,
            allowed_paths: dto.allowed_paths.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PathMappingDto> for PathMapping {
    fn from(dto: PathMappingDto) -> Self {
        Self(dto.0, dto.1.into())
    }
}

impl TryFrom<LoadConfigDto> for LoadConfig {
    type Error = ManifestError;

    fn try_from(dto: LoadConfigDto) -> Result<Self, Self::Error> {
        match dto {
            LoadConfigDto::Extism { file, memory_limit } => Ok(Self::Extism {
                file: file.try_into()?,
                memory_limit,
            }),
        }
    }
}

impl TryFrom<ApiConfigDto> for ApiConfig {
    type Error = ManifestError;

    fn try_from(dto: ApiConfigDto) -> Result<Self, Self::Error> {
        let version = semver::VersionReq::from_str(&dto.version).map_err(|e| {
            ManifestError::InvalidSemver {
                field: "api.version".to_owned(),
                value: dto.version.clone(),
                error: e.to_string(),
            }
        })?;

        Ok(Self {
            version,
            features: dto.features,
            host_functions: dto.host_functions,
        })
    }
}

// ── Domain → DTO (infallible) ──

impl From<PluginManifest> for PluginManifestDto {
    fn from(v: PluginManifest) -> Self {
        Self {
            dollar_schema: None,
            metadata: v.metadata.into(),
            runtime: v.runtime.into(),
            load: v.load.into(),
            api: v.api.into(),
        }
    }
}

impl From<PluginMetadata> for PluginMetadataDto {
    fn from(v: PluginMetadata) -> Self {
        Self {
            id: v.id,
            name: v.name,
            version: v.version.to_string(),
            description: v.description,
            authors: v.authors,
            license: v.license,
        }
    }
}

impl From<RuntimeConfig> for RuntimeConfigDto {
    fn from(v: RuntimeConfig) -> Self {
        Self {
            allowed_hosts: v.allowed_hosts,
            allowed_paths: v.allowed_paths.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PathMapping> for PathMappingDto {
    fn from(v: PathMapping) -> Self {
        Self(v.0, v.1.to_string_lossy().to_string())
    }
}

impl From<LoadConfig> for LoadConfigDto {
    fn from(v: LoadConfig) -> Self {
        match v {
            LoadConfig::Extism { file, memory_limit } => Self::Extism {
                file: file.into(),
                memory_limit,
            },
        }
    }
}

impl From<ApiConfig> for ApiConfigDto {
    fn from(v: ApiConfig) -> Self {
        Self {
            version: v.version.to_string(),
            features: v.features,
            host_functions: v.host_functions,
        }
    }
}

impl From<&LoadConfig> for LoadConfigType {
    fn from(config: &LoadConfig) -> Self {
        match config {
            LoadConfig::Extism { .. } => LoadConfigType::Extism,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn parse(api: &str) -> PluginManifest {
        let json = format!(
            r#"{{
                "metadata": {{ "id": "test", "name": "Test", "version": "0.1.0" }},
                "runtime": {{}},
                "load": {{ "type": "extism", "file": "test.wasm" }},
                "api": {api}
            }}"#
        );

        serde_json::from_str::<PluginManifestDto>(&json)
            .expect("manifest should parse")
            .try_into()
            .expect("manifest should map")
    }

    /// Manifests written before `api.hostFunctions` existed must keep parsing — and must stay
    /// distinguishable from an explicit empty declaration.
    #[rstest]
    #[case::absent_field_reads_as_undeclared(r#"{ "version": "0.2.0" }"#, None)]
    #[case::empty_list_reads_as_declaring_none(
        r#"{ "version": "0.2.0", "hostFunctions": [] }"#,
        Some(vec![])
    )]
    #[case::named_functions(
        r#"{ "version": "0.2.0", "hostFunctions": ["log", "http_get"] }"#,
        Some(vec!["log".to_owned(), "http_get".to_owned()])
    )]
    fn should_read_the_host_function_declaration(
        #[case] api: &str,
        #[case] expected: Option<Vec<String>>,
    ) {
        let manifest = parse(api);

        assert_eq!(manifest.api.host_functions, expected);
    }

    #[test]
    fn should_not_write_a_host_function_declaration_when_none_was_made() {
        let manifest = parse(r#"{ "version": "0.2.0" }"#);

        let json = serde_json::to_value(PluginManifestDto::from(manifest)).expect("serialize");

        assert!(json["api"].get("hostFunctions").is_none(), "{json}");
    }
}

#[cfg(test)]
mod load_file_tests {
    use super::*;

    #[test]
    fn should_reject_a_load_file_outside_the_plugin_dir_when_mapping_the_manifest() {
        let json = r#"{
            "metadata": { "id": "test", "name": "Test", "version": "0.1.0" },
            "runtime": {},
            "load": { "type": "extism", "file": "../outside.wasm" },
            "api": { "version": "0.2.0" }
        }"#;

        let result = PluginManifest::try_from(
            serde_json::from_str::<PluginManifestDto>(json).expect("manifest should parse"),
        );

        assert!(matches!(result, Err(ManifestError::InvalidLoadFile(_))));
    }
}
