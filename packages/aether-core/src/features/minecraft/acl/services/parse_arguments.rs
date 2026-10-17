use crate::features::minecraft::{
    MinecraftDomainError, TEMPORARY_REPLACE_CHAR, acl::models::vanilla,
};

use crate::features::minecraft::acl::services::rules::parse_rules;

/// Runs `transform` on a template with its spaces swapped for the temporary char.
fn transform_template<F>(template: &str, transform: &F) -> Result<String, MinecraftDomainError>
where
    F: Fn(&str) -> Result<String, MinecraftDomainError>,
{
    transform(&template.replace(' ', TEMPORARY_REPLACE_CHAR))
}

pub fn parse_arguments<F>(
    arguments: &[vanilla::Argument],
    parsed_arguments: &mut Vec<String>,
    parse_function: F,
    java_arch: &str,
) -> Result<(), MinecraftDomainError>
where
    F: Fn(&str) -> Result<String, MinecraftDomainError>,
{
    for argument in arguments {
        match argument {
            vanilla::Argument::Normal(template) => {
                let result = transform_template(template, &parse_function)?;
                parsed_arguments.extend(result.split(TEMPORARY_REPLACE_CHAR).map(str::to_owned));
            }
            vanilla::Argument::Ruled { rules, value } => {
                if !parse_rules(rules, java_arch, true) {
                    continue;
                }
                match value {
                    // Ruled values are single arguments already: pass them through verbatim
                    vanilla::ArgumentValue::Single(template) => {
                        parsed_arguments.push(parse_function(template)?);
                    }
                    vanilla::ArgumentValue::Many(templates) => {
                        for template in templates {
                            parsed_arguments.push(parse_function(template)?);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::minecraft::vanilla::{
        Argument, ArgumentValue, FeatureRule, Os, OsRule, Rule, RuleAction,
    };

    #[allow(clippy::unnecessary_wraps)]
    fn identity(s: &str) -> Result<String, MinecraftDomainError> {
        Ok(s.to_string())
    }

    fn run<F>(args: &[Argument], f: F) -> Result<Vec<String>, MinecraftDomainError>
    where
        F: Fn(&str) -> Result<String, MinecraftDomainError>,
    {
        let mut out = Vec::new();
        parse_arguments(args, &mut out, f, "x64")?;
        Ok(out)
    }

    fn normal(s: &str) -> Argument {
        Argument::Normal(s.to_string())
    }

    fn allow_all() -> Vec<Rule> {
        vec![Rule {
            action: RuleAction::Allow,
            os: None,
            features: None,
        }]
    }

    fn never_matching_os_rule() -> Rule {
        Rule {
            action: RuleAction::Allow,
            os: Some(OsRule {
                name: Some(Os::Unknown),
                version: None,
                arch: None,
            }),
            features: None,
        }
    }

    fn failing(_: &str) -> Result<String, MinecraftDomainError> {
        Err(MinecraftDomainError::ParseFailed {
            reason: "boom".to_string(),
        })
    }

    #[test]
    fn empty_input_produces_nothing() {
        assert_eq!(run(&[], identity).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn appends_to_existing_output() {
        let mut out = vec!["first".to_string()];
        parse_arguments(&[normal("second")], &mut out, identity, "x64").unwrap();
        assert_eq!(out, vec!["first", "second"]);
    }

    #[test]
    fn normal_without_spaces_is_single_argument() {
        assert_eq!(
            run(&[normal("--demo"), normal("-Xss1M")], identity).unwrap(),
            vec!["--demo", "-Xss1M"]
        );
    }

    #[test]
    fn normal_with_spaces_is_split_on_spaces() {
        assert_eq!(
            run(&[normal("--width 800 --height 600")], identity).unwrap(),
            vec!["--width", "800", "--height", "600"]
        );
    }

    #[test]
    fn normal_parse_function_sees_spaces_replaced_by_temporary_char() {
        let seen = std::cell::RefCell::new(Vec::new());
        run(&[normal("a b")], |s| {
            seen.borrow_mut().push(s.to_string());
            Ok(s.to_string())
        })
        .unwrap();
        assert_eq!(
            seen.into_inner(),
            vec![format!("a{TEMPORARY_REPLACE_CHAR}b")]
        );
    }

    #[test]
    fn normal_placeholder_value_with_spaces_is_not_split() {
        let out = run(&[normal("--dir ${dir}")], |s| {
            Ok(s.replace("${dir}", "C:/My Games/mc"))
        })
        .unwrap();
        assert_eq!(out, vec!["--dir", "C:/My Games/mc"]);
    }

    #[test]
    fn normal_empty_string_gives_one_empty_argument() {
        assert_eq!(run(&[normal("")], identity).unwrap(), vec![""]);
    }

    #[test]
    fn normal_consecutive_spaces_give_empty_arguments() {
        assert_eq!(
            run(&[normal("a  b")], identity).unwrap(),
            vec!["a", "", "b"]
        );
    }

    #[test]
    fn ruled_single_allowed_is_added() {
        let arg = Argument::Ruled {
            rules: allow_all(),
            value: ArgumentValue::Single("--flag".to_string()),
        };
        assert_eq!(run(&[arg], identity).unwrap(), vec!["--flag"]);
    }

    #[test]
    fn ruled_single_denied_is_skipped_and_parse_function_not_called() {
        let arg = Argument::Ruled {
            rules: vec![never_matching_os_rule()],
            value: ArgumentValue::Single("--flag".to_string()),
        };
        let calls = std::cell::Cell::new(0);
        let out = run(&[arg], |s| {
            calls.set(calls.get() + 1);
            Ok(s.to_string())
        })
        .unwrap();
        assert!(out.is_empty());
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn ruled_many_allowed_adds_each_value_in_order() {
        let arg = Argument::Ruled {
            rules: allow_all(),
            value: ArgumentValue::Many(vec!["--a".to_string(), "--b".to_string()]),
        };
        assert_eq!(run(&[arg], identity).unwrap(), vec!["--a", "--b"]);
    }

    #[test]
    fn ruled_many_denied_is_skipped() {
        let arg = Argument::Ruled {
            rules: vec![never_matching_os_rule()],
            value: ArgumentValue::Many(vec!["--a".to_string(), "--b".to_string()]),
        };
        assert!(run(&[arg], identity).unwrap().is_empty());
    }

    #[test]
    fn ruled_with_empty_rules_is_allowed() {
        let arg = Argument::Ruled {
            rules: vec![],
            value: ArgumentValue::Single("--flag".to_string()),
        };
        assert_eq!(run(&[arg], identity).unwrap(), vec!["--flag"]);
    }

    #[test]
    fn ruled_feature_rule_for_demo_user_is_currently_skipped() {
        let arg = Argument::Ruled {
            rules: vec![Rule {
                action: RuleAction::Allow,
                os: None,
                features: Some(FeatureRule {
                    is_demo_user: Some(true),
                    has_custom_resolution: None,
                    has_quick_plays_support: None,
                    is_quick_play_singleplayer: None,
                    is_quick_play_multiplayer: None,
                    is_quick_play_realms: None,
                }),
            }],
            value: ArgumentValue::Single("--demo".to_string()),
        };
        assert!(run(&[arg], identity).unwrap().is_empty());
    }

    #[test]
    fn ruled_single_with_spaces_is_passed_verbatim() {
        let arg = Argument::Ruled {
            rules: allow_all(),
            value: ArgumentValue::Single("a b".to_string()),
        };
        assert_eq!(run(&[arg], identity).unwrap(), vec!["a b"]);
    }

    #[test]
    fn ruled_many_with_spaces_is_passed_verbatim_per_value() {
        let arg = Argument::Ruled {
            rules: allow_all(),
            value: ArgumentValue::Many(vec!["-x".to_string(), "a b".to_string()]),
        };
        assert_eq!(
            run(&[arg], identity).unwrap(),
            vec!["-x".to_string(), "a b".to_string()]
        );
    }

    #[test]
    fn ruled_parse_function_sees_value_unchanged() {
        let seen = std::cell::RefCell::new(Vec::new());
        let arg = Argument::Ruled {
            rules: allow_all(),
            value: ArgumentValue::Many(vec!["-Dos.name=Windows 10".to_string()]),
        };
        let out = run(&[arg], |s| {
            seen.borrow_mut().push(s.to_string());
            Ok(s.to_string())
        })
        .unwrap();
        assert_eq!(out, vec!["-Dos.name=Windows 10"]);
        assert_eq!(seen.into_inner(), vec!["-Dos.name=Windows 10"]);
    }

    #[test]
    fn ruled_placeholder_value_with_spaces_is_not_split() {
        let arg = Argument::Ruled {
            rules: allow_all(),
            value: ArgumentValue::Single("${dir}".to_string()),
        };
        let out = run(&[arg], |s| Ok(s.replace("${dir}", "My Dir"))).unwrap();
        assert_eq!(out, vec!["My Dir"]);
    }

    #[test]
    fn mixed_arguments_keep_order() {
        let args = vec![
            normal("-a 1"),
            Argument::Ruled {
                rules: allow_all(),
                value: ArgumentValue::Many(vec!["-b".to_string(), "2".to_string()]),
            },
            Argument::Ruled {
                rules: vec![never_matching_os_rule()],
                value: ArgumentValue::Single("-skipped".to_string()),
            },
            normal("-c"),
        ];
        assert_eq!(
            run(&args, identity).unwrap(),
            vec!["-a", "1", "-b", "2", "-c"]
        );
    }

    #[test]
    fn error_from_normal_is_propagated() {
        let err = run(&[normal("x")], failing).unwrap_err();
        assert!(
            matches!(err, MinecraftDomainError::ParseFailed { ref reason } if reason == "boom")
        );
    }

    #[test]
    fn error_from_ruled_single_and_many_is_propagated() {
        for value in [
            ArgumentValue::Single("x".to_string()),
            ArgumentValue::Many(vec!["x".to_string()]),
        ] {
            let arg = Argument::Ruled {
                rules: allow_all(),
                value,
            };
            assert!(run(&[arg], failing).is_err());
        }
    }

    #[test]
    fn error_from_denied_ruled_argument_is_not_triggered() {
        let arg = Argument::Ruled {
            rules: vec![never_matching_os_rule()],
            value: ArgumentValue::Single("x".to_string()),
        };
        assert!(run(&[arg], failing).unwrap().is_empty());
    }

    #[test]
    fn error_stops_processing_but_keeps_earlier_output() {
        let mut out = Vec::new();
        let res = parse_arguments(
            &[normal("ok"), normal("bad"), normal("never")],
            &mut out,
            |s| {
                if s == "bad" {
                    failing(s)
                } else {
                    Ok(s.to_string())
                }
            },
            "x64",
        );
        assert!(res.is_err());
        assert_eq!(out, vec!["ok"]);
    }
}
