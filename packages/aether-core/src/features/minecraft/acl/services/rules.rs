use regex::Regex;

use crate::features::minecraft::acl::models::vanilla;

/// Result of evaluating a single rule.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Vote {
    Allow,
    Deny,
    Abstain,
}

pub fn parse_rules(rules: &[vanilla::Rule], java_version: &str, minecraft_updated: bool) -> bool {
    let mut votes: Vec<Vote> = rules
        .iter()
        .map(|rule| vote(rule, java_version, minecraft_updated))
        .collect();

    if rules
        .iter()
        .all(|rule| matches!(rule.action, vanilla::RuleAction::Disallow))
    {
        votes.push(Vote::Allow);
    }

    !votes.contains(&Vote::Deny) && votes.contains(&Vote::Allow)
}

fn vote(rule: &vanilla::Rule, java_arch: &str, minecraft_updated: bool) -> Vote {
    let condition = match (&rule.os, &rule.features) {
        (Some(os), _) => parse_os_rule(os, java_arch, minecraft_updated),
        (None, Some(features)) => features_hold(features),
        // Unconditional rules always vote positively, whatever their action.
        (None, None) => return Vote::Allow,
    };

    match (&rule.action, condition) {
        (vanilla::RuleAction::Allow, true) => Vote::Allow,
        (vanilla::RuleAction::Allow, false) | (vanilla::RuleAction::Disallow, true) => Vote::Deny,
        (vanilla::RuleAction::Disallow, false) => Vote::Abstain,
    }
}

fn features_hold(features: &vanilla::FeatureRule) -> bool {
    features.is_demo_user == Some(false)
        || features.has_custom_resolution == Some(true)
        || features.has_quick_plays_support == Some(false)
        || features.is_quick_play_multiplayer == Some(false)
        || features.is_quick_play_realms == Some(false)
        || features.is_quick_play_singleplayer == Some(false)
}

// Platform rule resolving
pub(super) fn parse_os_rule(
    rule: &vanilla::OsRule,
    java_arch: &str,
    // Minecraft updated over 1.18.2 (supports MacOS Natively)
    minecraft_updated: bool,
) -> bool {
    let arch_ok = rule
        .arch
        .as_deref()
        .is_none_or(|arch| arch != "x86" && arch != "arm");

    let name_ok = rule.name.as_ref().is_none_or(|name| {
        *name == vanilla::Os::native_arch(java_arch)
            || (minecraft_updated && *name == vanilla::Os::native())
    });

    let version_ok = rule.version.as_deref().is_none_or(|pattern| {
        Regex::new(pattern).map_or(true, |re| {
            re.is_match(&sys_info::os_release().unwrap_or_default())
        })
    });

    arch_ok && name_ok && version_ok
}

#[macro_export]
macro_rules! processor_rules {
  ($dest:expr_2021; $($name:literal : client => $client:expr_2021, server => $server:expr_2021;)+) => {
      $(std::collections::HashMap::insert(
          $dest,
          String::from($name),
          $crate::features::minecraft::modded::SidedDataEntry {
              client: String::from($client),
              server: String::from($server),
          },
      );)+
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::minecraft::vanilla::{FeatureRule, Os, OsRule, Rule, RuleAction};
    use std::collections::HashMap;

    fn plain(action: RuleAction) -> Rule {
        Rule {
            action,
            os: None,
            features: None,
        }
    }

    fn os_rule(action: RuleAction, os: OsRule) -> Rule {
        Rule {
            action,
            os: Some(os),
            features: None,
        }
    }

    fn os_name(name: Os) -> OsRule {
        OsRule {
            name: Some(name),
            version: None,
            arch: None,
        }
    }

    fn feature_rule(action: RuleAction, features: FeatureRule) -> Rule {
        Rule {
            action,
            os: None,
            features: Some(features),
        }
    }

    fn no_features() -> FeatureRule {
        FeatureRule {
            is_demo_user: None,
            has_custom_resolution: None,
            has_quick_plays_support: None,
            is_quick_play_singleplayer: None,
            is_quick_play_multiplayer: None,
            is_quick_play_realms: None,
        }
    }

    /// An OS name that matches the host for any x86-like Java architecture
    fn matching() -> OsRule {
        os_name(Os::native())
    }

    /// An OS name that never matches the host
    fn not_matching() -> OsRule {
        os_name(Os::Unknown)
    }

    // ---- parse_rules: combination logic ----

    #[test]
    fn empty_rules_are_allowed() {
        assert!(parse_rules(&[], "x64", true));
        assert!(parse_rules(&[], "x64", false));
    }

    #[test]
    fn allow_without_conditions_is_allowed() {
        assert!(parse_rules(&[plain(RuleAction::Allow)], "x64", true));
    }

    #[test]
    fn disallow_without_conditions_is_currently_allowed() {
        // An unconditional rule resolves to Some(true) regardless of its action
        assert!(parse_rules(&[plain(RuleAction::Disallow)], "x64", true));
    }

    #[test]
    fn allow_with_matching_os_is_allowed() {
        let rules = [os_rule(RuleAction::Allow, matching())];
        assert!(parse_rules(&rules, "x64", true));
        assert!(parse_rules(&rules, "x64", false));
    }

    #[test]
    fn allow_with_non_matching_os_is_denied() {
        let rules = [os_rule(RuleAction::Allow, not_matching())];
        assert!(!parse_rules(&rules, "x64", true));
        assert!(!parse_rules(&rules, "x64", false));
    }

    #[test]
    fn disallow_with_matching_os_is_denied() {
        let rules = [os_rule(RuleAction::Disallow, matching())];
        assert!(!parse_rules(&rules, "x64", true));
        assert!(!parse_rules(&rules, "x64", false));
    }

    #[test]
    fn disallow_with_non_matching_os_is_allowed() {
        let rules = [os_rule(RuleAction::Disallow, not_matching())];
        assert!(parse_rules(&rules, "x64", true));
        assert!(parse_rules(&rules, "x64", false));
    }

    #[test]
    fn all_disallow_one_matching_denies_even_if_others_do_not_match() {
        let rules = [
            os_rule(RuleAction::Disallow, not_matching()),
            os_rule(RuleAction::Disallow, matching()),
        ];
        assert!(!parse_rules(&rules, "x64", true));
    }

    #[test]
    fn mixed_allow_unconditional_and_disallow_matching_is_denied() {
        let rules = [
            plain(RuleAction::Allow),
            os_rule(RuleAction::Disallow, matching()),
        ];
        assert!(!parse_rules(&rules, "x64", true));
    }

    #[test]
    fn mixed_allow_unconditional_and_disallow_non_matching_is_allowed() {
        let rules = [
            plain(RuleAction::Allow),
            os_rule(RuleAction::Disallow, not_matching()),
        ];
        assert!(parse_rules(&rules, "x64", true));
    }

    #[test]
    fn mixed_allow_matching_and_disallow_non_matching_is_allowed() {
        let rules = [
            os_rule(RuleAction::Allow, matching()),
            os_rule(RuleAction::Disallow, not_matching()),
        ];
        assert!(parse_rules(&rules, "x64", true));
    }

    #[test]
    fn mixed_allow_non_matching_and_disallow_non_matching_is_denied() {
        let rules = [
            os_rule(RuleAction::Allow, not_matching()),
            os_rule(RuleAction::Disallow, not_matching()),
        ];
        assert!(!parse_rules(&rules, "x64", true));
    }

    #[test]
    fn any_failing_allow_denies_the_whole_list() {
        let rules = [
            os_rule(RuleAction::Allow, matching()),
            os_rule(RuleAction::Allow, not_matching()),
        ];
        assert!(!parse_rules(&rules, "x64", true));
    }

    // ---- parse_rules: features ----

    fn allow_features(f: FeatureRule) -> bool {
        parse_rules(&[feature_rule(RuleAction::Allow, f)], "x64", true)
    }

    #[test]
    fn features_all_unset_are_denied() {
        assert!(!allow_features(no_features()));
    }

    #[test]
    fn features_demo_user_true_is_denied_false_is_allowed() {
        assert!(!allow_features(FeatureRule {
            is_demo_user: Some(true),
            ..no_features()
        }));
        assert!(allow_features(FeatureRule {
            is_demo_user: Some(false),
            ..no_features()
        }));
    }

    #[test]
    fn features_custom_resolution_true_is_allowed_false_is_denied() {
        assert!(allow_features(FeatureRule {
            has_custom_resolution: Some(true),
            ..no_features()
        }));
        assert!(!allow_features(FeatureRule {
            has_custom_resolution: Some(false),
            ..no_features()
        }));
    }

    #[test]
    fn features_quick_play_flags_true_are_denied_false_are_allowed() {
        let setters: [fn(&mut FeatureRule, Option<bool>); 4] = [
            |f, v| f.has_quick_plays_support = v,
            |f, v| f.is_quick_play_singleplayer = v,
            |f, v| f.is_quick_play_multiplayer = v,
            |f, v| f.is_quick_play_realms = v,
        ];
        for set in setters {
            let mut on = no_features();
            set(&mut on, Some(true));
            assert!(!allow_features(on));

            let mut off = no_features();
            set(&mut off, Some(false));
            assert!(allow_features(off));
        }
    }

    #[test]
    fn features_any_single_truthy_condition_is_enough() {
        assert!(allow_features(FeatureRule {
            is_demo_user: Some(true),
            has_custom_resolution: Some(true),
            ..no_features()
        }));
    }

    #[test]
    fn disallow_feature_rule_inverts() {
        // Condition holds (custom resolution) -> disallowed
        let holds = [feature_rule(
            RuleAction::Disallow,
            FeatureRule {
                has_custom_resolution: Some(true),
                ..no_features()
            },
        )];
        assert!(!parse_rules(&holds, "x64", true));

        // Condition does not hold (nothing set) -> allowed
        let not_holds = [feature_rule(RuleAction::Disallow, no_features())];
        assert!(parse_rules(&not_holds, "x64", true));
    }

    #[test]
    fn rule_with_os_ignores_features() {
        let rule = Rule {
            action: RuleAction::Allow,
            os: Some(matching()),
            features: Some(no_features()),
        };
        assert!(parse_rules(&[rule], "x64", true));

        let rule = Rule {
            action: RuleAction::Allow,
            os: Some(not_matching()),
            features: Some(FeatureRule {
                has_custom_resolution: Some(true),
                ..no_features()
            }),
        };
        assert!(!parse_rules(&[rule], "x64", true));
    }

    #[test]
    fn features_do_not_depend_on_minecraft_updated_or_arch() {
        let rules = [feature_rule(
            RuleAction::Allow,
            FeatureRule {
                has_custom_resolution: Some(true),
                ..no_features()
            },
        )];
        for arch in ["x64", "aarch64", "arm"] {
            for updated in [true, false] {
                assert!(parse_rules(&rules, arch, updated));
            }
        }
    }

    // ---- parse_os_rule ----

    #[test]
    fn os_rule_empty_matches() {
        let rule = OsRule {
            name: None,
            version: None,
            arch: None,
        };
        assert!(parse_os_rule(&rule, "x64", true));
        assert!(parse_os_rule(&rule, "x64", false));
    }

    #[test]
    fn os_rule_arch_x86_and_arm_never_match() {
        for (arch, expected) in [
            ("x86", false),
            ("arm", false),
            ("x64", true),
            ("aarch64", true),
        ] {
            let rule = OsRule {
                name: None,
                version: None,
                arch: Some(arch.to_string()),
            };
            assert_eq!(parse_os_rule(&rule, "x64", true), expected, "arch {arch}");
            assert_eq!(parse_os_rule(&rule, "x64", false), expected, "arch {arch}");
        }
    }

    #[test]
    fn os_rule_arch_is_independent_of_java_arch() {
        let rule = OsRule {
            name: None,
            version: None,
            arch: Some("x86".to_string()),
        };
        assert!(!parse_os_rule(&rule, "aarch64", true));
    }

    #[test]
    fn os_rule_native_name_matches_for_x64() {
        assert!(parse_os_rule(&matching(), "x64", true));
        assert!(parse_os_rule(&matching(), "x64", false));
    }

    #[test]
    fn os_rule_unknown_name_does_not_match() {
        assert!(!parse_os_rule(&not_matching(), "x64", true));
        assert!(!parse_os_rule(&not_matching(), "x64", false));
    }

    #[test]
    fn os_rule_native_name_with_aarch64_java_depends_on_minecraft_updated() {
        // updated: the plain native OS name still matches
        assert!(parse_os_rule(&matching(), "aarch64", true));
        // not updated: only the arch-specific variant matches
        assert!(!parse_os_rule(&matching(), "aarch64", false));
    }

    #[test]
    fn os_rule_arch_variant_name_matches_only_with_matching_java_arch() {
        let variant = os_name(Os::native_arch("aarch64"));
        for updated in [true, false] {
            assert!(parse_os_rule(&variant, "aarch64", updated));
            assert!(!parse_os_rule(&variant, "x64", updated));
        }
    }

    #[test]
    fn os_rule_linux_arm_names_use_same_branch_as_other_names() {
        // The `name != LinuxArm64 || name != LinuxArm32` guard is always true,
        // so these behave exactly like any other name.
        let arm32 = os_name(Os::LinuxArm32);
        let arm64 = os_name(Os::LinuxArm64);
        let expect32 = Os::native() == Os::LinuxArm32 || Os::native_arch("arm") == Os::LinuxArm32;
        let expect64 =
            Os::native() == Os::LinuxArm64 || Os::native_arch("aarch64") == Os::LinuxArm64;
        assert_eq!(parse_os_rule(&arm32, "arm", true), expect32);
        assert_eq!(parse_os_rule(&arm64, "aarch64", true), expect64);
        assert_eq!(
            parse_os_rule(&arm64, "aarch64", false),
            Os::native_arch("aarch64") == Os::LinuxArm64
        );
    }

    #[test]
    fn os_rule_version_regex_matching_anything_keeps_match() {
        let rule = OsRule {
            version: Some(".*".to_string()),
            ..matching()
        };
        assert!(parse_os_rule(&rule, "x64", true));
    }

    #[test]
    fn os_rule_version_regex_never_matching_rejects() {
        let rule = OsRule {
            name: None,
            version: Some("a^b".to_string()),
            arch: None,
        };
        assert!(!parse_os_rule(&rule, "x64", true));
    }

    #[test]
    fn os_rule_invalid_version_regex_is_ignored() {
        let rule = OsRule {
            name: None,
            version: Some("(".to_string()),
            arch: None,
        };
        assert!(parse_os_rule(&rule, "x64", true));
    }

    #[test]
    fn os_rule_conditions_are_anded() {
        let rule = OsRule {
            name: Some(Os::native()),
            version: Some(".*".to_string()),
            arch: Some("x86".to_string()),
        };
        assert!(!parse_os_rule(&rule, "x64", true));
    }

    // ---- processor_rules! ----

    #[test]
    fn processor_rules_macro_inserts_sided_entries() {
        let mut data: HashMap<String, crate::features::minecraft::modded::SidedDataEntry> =
            HashMap::new();
        let jar = String::from("client.jar");
        crate::processor_rules! {
            &mut data;
            "SIDE":
                client => "client",
                server => "";
            "MINECRAFT_JAR":
                client => jar.clone(),
                server => "srv";
        }
        assert_eq!(data.len(), 2);
        assert_eq!(data["SIDE"].client, "client");
        assert_eq!(data["SIDE"].server, "");
        assert_eq!(data["MINECRAFT_JAR"].client, "client.jar");
        assert_eq!(data["MINECRAFT_JAR"].server, "srv");
    }

    #[test]
    fn processor_rules_macro_overwrites_existing_keys() {
        let mut data = HashMap::new();
        crate::processor_rules! {
            &mut data;
            "SIDE": client => "old", server => "";
        }
        crate::processor_rules! {
            &mut data;
            "SIDE": client => "new", server => "";
        }
        assert_eq!(data.len(), 1);
        assert_eq!(data["SIDE"].client, "new");
    }
}
