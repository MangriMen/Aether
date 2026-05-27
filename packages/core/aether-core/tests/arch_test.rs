use rust_arkitect::dsl::architectural_rules::{ArchitecturalRules, SubjectInjectableRuleBuilder};
use rust_arkitect::dsl::arkitect::Arkitect;
use rust_arkitect::dsl::project::Project;
use rust_arkitect::rule::Rule;
use rust_arkitect::rust_file::RustFile;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::MAIN_SEPARATOR;

pub struct IsolateLayer {
    forbidden_patterns: Vec<&'static str>,
}

impl IsolateLayer {
    pub fn forbidden_to_access(
        patterns: Vec<&'static str>,
    ) -> Box<dyn SubjectInjectableRuleBuilder> {
        Box::new(IsolateLayer {
            forbidden_patterns: patterns,
        })
    }
}

impl SubjectInjectableRuleBuilder for IsolateLayer {
    fn for_subject(&self, subject: &str) -> Box<dyn Rule> {
        Box::new(StrictLayerRule {
            layer_folder_name: subject.to_string(),
            forbidden_imports: self
                .forbidden_patterns
                .iter()
                .map(|&s| s.to_string())
                .collect(),
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
        // --- Auth Feature Rules ---
        .rules_for_module("domain")
            .it(IsolateLayer::forbidden_to_access(vec![
                "features::auth::app",
                "features::auth::infra",
            ]))
        .rules_for_module("app")
            .it(IsolateLayer::forbidden_to_access(vec![
                "features::auth::infra",
            ]))
        .build();

    let result = Arkitect::ensure_that(project).complies_with(rules);
    assert!(result.is_ok(), "Architectural boundaries violated!");
}
