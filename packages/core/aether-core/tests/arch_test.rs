use rust_arkitect::dsl::architectural_rules::{ArchitecturalRules, SubjectInjectableRuleBuilder};
use rust_arkitect::dsl::arkitect::Arkitect;
use rust_arkitect::dsl::project::Project;
use rust_arkitect::rule::Rule;
use rust_arkitect::rust_file::RustFile;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::MAIN_SEPARATOR;

/// All features in the aether-core crate.
const ALL_FEATURES: &[&str] = &[
    "auth",
    "events",
    "file_watcher",
    "instance",
    "java",
    "minecraft",
    "plugins",
    "process",
    "settings",
];

pub struct IsolateLayer {
    forbidden_patterns: Vec<String>,
}

impl IsolateLayer {
    /// Creates a rule builder that forbids files in a layer from importing the given patterns.
    pub fn forbidden_to_access(
        patterns: Vec<&'static str>,
    ) -> Box<dyn SubjectInjectableRuleBuilder> {
        Box::new(IsolateLayer {
            forbidden_patterns: patterns.into_iter().map(String::from).collect(),
        })
    }

    /// Forbids the domain layer from importing `app` or `infra` of ANY feature.
    fn for_domain() -> Box<dyn SubjectInjectableRuleBuilder> {
        let mut patterns = Vec::new();
        for feature in ALL_FEATURES {
            patterns.push(format!("features::{feature}::app"));
            patterns.push(format!("features::{feature}::infra"));
        }
        Box::new(IsolateLayer {
            forbidden_patterns: patterns,
        })
    }

    /// Forbids the app layer from importing `infra` of ANY feature.
    fn for_app() -> Box<dyn SubjectInjectableRuleBuilder> {
        let patterns: Vec<String> = ALL_FEATURES
            .iter()
            .map(|feature| format!("features::{feature}::infra"))
            .collect();
        Box::new(IsolateLayer {
            forbidden_patterns: patterns,
        })
    }
}

impl SubjectInjectableRuleBuilder for IsolateLayer {
    fn for_subject(&self, subject: &str) -> Box<dyn Rule> {
        Box::new(StrictLayerRule {
            layer_folder_name: subject.to_string(),
            forbidden_imports: self.forbidden_patterns.clone(),
        })
    }
}

struct StrictLayerRule {
    layer_folder_name: String,
    forbidden_imports: Vec<String>,
}

impl Display for StrictLayerRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Layer '{}' must not contain imports from {:?}",
            self.layer_folder_name, self.forbidden_imports
        )
    }
}

impl Rule for StrictLayerRule {
    fn is_applicable(&self, file: &RustFile) -> bool {
        let path_str = file.path.clone();
        // Check if the file is physically located inside the target layer folder (e.g., /domain/)
        path_str.contains(&format!(
            "{}{}{}",
            MAIN_SEPARATOR, self.layer_folder_name, MAIN_SEPARATOR
        ))
    }

    fn apply(&self, file: &RustFile) -> Result<(), String> {
        let content = fs::read_to_string(&file.path)
            .map_err(|e| format!("Failed to read file {:?}: {}", file.path, e))?;

        for forbidden in &self.forbidden_imports {
            if content.contains(forbidden) {
                return Err(format!(
                    "Architecture violation: import '{forbidden}' is forbidden inside the '{}' layer.",
                    self.layer_folder_name
                ));
            }
        }
        Ok(())
    }
}

#[test]
fn test_hexagonal_architecture_boundaries() {
    Arkitect::init_logger();
    let project = Project::from_current_crate();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        // === Domain Layer: must NOT import `app` or `infra` of ANY feature ===
        .rules_for_module("domain")
            .it(IsolateLayer::for_domain())
        // === App Layer: must NOT import `infra` of ANY feature ===
        .rules_for_module("app")
            .it(IsolateLayer::for_app())
        .build();

    let result = Arkitect::ensure_that(project).complies_with(rules);
    assert!(result.is_ok(), "Architectural boundaries violated!");
}
