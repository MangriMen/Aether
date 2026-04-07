use register_schema::get_all_schemas;
use std::fs;
use std::path::{Path, PathBuf};

// Paths relative to workspace
const PLUGIN_API_PATH: &[&str] = &["aether-core-plugin-api", "src", "v0", "schemas"];
const INTERNAL_PATH: &[&str] = &["spec", "internal-schemas"];
const DEFAULT_PATH: &[&str] = &["spec", "json-schema"];

#[test]
fn sync_all_schemas() {
    // Ensure is linked
    let _ = aether_core::features::plugins::PluginCapabilities::default();

    let schemas = get_all_schemas();
    assert!(
        !schemas.is_empty(),
        "No schemas registered! Ensure aether-core is linked and types use #[derive(RegisterSchema)]"
    );

    let update_mode = std::env::var("UPDATE_SCHEMAS").is_ok();

    for entry in schemas {
        let target_dir = get_target_dir(entry.category);

        if !target_dir.exists() {
            fs::create_dir_all(&target_dir).expect("Failed to create target directory");
        }

        let file_name = format!("{}.schema.json", to_kebab_case(entry.name));
        let file_path = target_dir.join(file_name);

        let schema_obj = (entry.schema)();
        let actual_json =
            serde_json::to_string_pretty(&schema_obj).expect("Failed to serialize schema");

        process_schema(&file_path, actual_json, update_mode);
    }
}

fn get_target_dir(category: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Workspace root

    let sub_path = match category {
        "plugin_api" => PLUGIN_API_PATH,
        "internal" => INTERNAL_PATH,
        _ => DEFAULT_PATH,
    };

    path.extend(sub_path);
    path
}

fn process_schema(path: &Path, actual_json: String, update_mode: bool) {
    if update_mode {
        fs::write(path, &actual_json).expect("Failed to write schema");
        println!("  [UPDATED] {:?}", path.file_name().unwrap());
    } else {
        let expected_json = fs::read_to_string(path).unwrap_or_else(|_| {
            panic!("\n\n[ERROR] Missing schema: {:?}\nRun 'UPDATE_SCHEMAS=1 cargo test' to generate.\n", path);
        });

        // Compare as Value, so that we don't fail on whitespace differences
        let actual_val: serde_json::Value = serde_json::from_str(&actual_json).unwrap();
        let expected_val: serde_json::Value = serde_json::from_str(&expected_json).unwrap();

        assert_eq!(
            actual_val, expected_val,
            "\n\n[ERROR] Schema mismatch in {:?}\nRun 'UPDATE_SCHEMAS=1 cargo test' to update.\n",
            path
        );
    }
}

fn to_kebab_case(s: &str) -> String {
    let mut kebab = String::with_capacity(s.len() + 2);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                kebab.push('-');
            }
            kebab.extend(ch.to_lowercase());
        } else {
            kebab.push(ch);
        }
    }
    kebab
}
