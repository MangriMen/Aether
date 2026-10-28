use regex::Regex;

use crate::features::minecraft::acl::models::vanilla;
use crate::shared::system::infra::get_os_version;

/// Result of evaluating a single rule.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Vote {
    Allow,
    Deny,
    Abstain,
}

pub fn parse_rules(rules: &[vanilla::Rule], java_arch: &str, minecraft_updated: bool) -> bool {
    let mut votes: Vec<Vote> = rules
        .iter()
        .map(|rule| vote(rule, java_arch, minecraft_updated))
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
    // A rule without conditions always holds; with several, all of them must hold.
    let condition = rule
        .os
        .as_ref()
        .is_none_or(|os| parse_os_rule(os, java_arch, minecraft_updated))
        && rule.features.as_ref().is_none_or(features_hold);

    match (&rule.action, condition) {
        (vanilla::RuleAction::Allow, true) => Vote::Allow,
        (vanilla::RuleAction::Allow, false) | (vanilla::RuleAction::Disallow, true) => Vote::Deny,
        (vanilla::RuleAction::Disallow, false) => Vote::Abstain,
    }
}

/// Whether at least one known feature flag is specified and every specified flag
/// matches the launcher's configuration. Flags the rule leaves unset are ignored.
/// A condition with no known flags (e.g. only flags unknown to us, dropped by serde)
/// does not hold.
fn features_hold(features: &vanilla::FeatureRule) -> bool {
    let any_specified = [
        features.is_demo_user,
        features.has_custom_resolution,
        features.has_quick_plays_support,
        features.is_quick_play_singleplayer,
        features.is_quick_play_multiplayer,
        features.is_quick_play_realms,
    ]
    .iter()
    .any(Option::is_some);
    let matches = |wanted: Option<bool>, actual: bool| wanted.is_none_or(|wanted| wanted == actual);

    any_specified
        && matches(features.is_demo_user, false)
        && matches(features.has_custom_resolution, true)
        && matches(features.has_quick_plays_support, false)
        && matches(features.is_quick_play_singleplayer, false)
        && matches(features.is_quick_play_multiplayer, false)
        && matches(features.is_quick_play_realms, false)
}

/// Whether an `os` condition of a rule holds for the given Java architecture.
pub(super) fn parse_os_rule(
    rule: &vanilla::OsRule,
    java_arch: &str,
    // `true` for game versions that run natively on every desktop OS the launcher supports
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

    let version_ok = rule
        .version
        .as_deref()
        .is_none_or(|pattern| version_matches(pattern, &get_os_version().unwrap_or_default()));

    arch_ok && name_ok && version_ok
}

/// Whether `version` matches the `os.version` pattern of a rule; an invalid pattern is ignored.
fn version_matches(pattern: &str, version: &str) -> bool {
    Regex::new(pattern).map_or(true, |re| re.is_match(version))
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
    use rstest::rstest;
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

    #[rstest]
    #[case::empty_rules_are_allowed(vec![], true)]
    #[case::allow_without_conditions_is_allowed(vec![plain(RuleAction::Allow)], true)]
    #[case::disallow_without_conditions_is_denied(vec![plain(RuleAction::Disallow)], false)]
    #[case::unconditional_disallow_denies_alongside_allow(
        vec![plain(RuleAction::Allow), plain(RuleAction::Disallow)],
        false
    )]
    #[case::allow_with_matching_os_is_allowed(
        vec![os_rule(RuleAction::Allow, matching())],
        true
    )]
    #[case::allow_with_non_matching_os_is_denied(
        vec![os_rule(RuleAction::Allow, not_matching())],
        false
    )]
    #[case::disallow_with_matching_os_is_denied(
        vec![os_rule(RuleAction::Disallow, matching())],
        false
    )]
    #[case::disallow_with_non_matching_os_is_allowed(
        vec![os_rule(RuleAction::Disallow, not_matching())],
        true
    )]
    #[case::all_disallow_one_matching_denies_even_if_others_do_not_match(
        vec![
            os_rule(RuleAction::Disallow, not_matching()),
            os_rule(RuleAction::Disallow, matching()),
        ],
        false
    )]
    #[case::mixed_allow_unconditional_and_disallow_matching_is_denied(
        vec![
            plain(RuleAction::Allow),
            os_rule(RuleAction::Disallow, matching()),
        ],
        false
    )]
    #[case::mixed_allow_unconditional_and_disallow_non_matching_is_allowed(
        vec![
            plain(RuleAction::Allow),
            os_rule(RuleAction::Disallow, not_matching()),
        ],
        true
    )]
    #[case::mixed_allow_matching_and_disallow_non_matching_is_allowed(
        vec![
            os_rule(RuleAction::Allow, matching()),
            os_rule(RuleAction::Disallow, not_matching()),
        ],
        true
    )]
    #[case::mixed_allow_non_matching_and_disallow_non_matching_is_denied(
        vec![
            os_rule(RuleAction::Allow, not_matching()),
            os_rule(RuleAction::Disallow, not_matching()),
        ],
        false
    )]
    #[case::any_failing_allow_denies_the_whole_list(
        vec![
            os_rule(RuleAction::Allow, matching()),
            os_rule(RuleAction::Allow, not_matching()),
        ],
        false
    )]
    fn parse_rules_combines_rules(
        #[case] rules: Vec<Rule>,
        #[case] expected: bool,
        #[values(true, false)] minecraft_updated: bool,
    ) {
        assert_eq!(parse_rules(&rules, "x64", minecraft_updated), expected);
    }

    // ---- parse_rules: features ----

    fn allow_features(f: FeatureRule) -> bool {
        parse_rules(&[feature_rule(RuleAction::Allow, f)], "x64", true)
    }

    #[test]
    fn rule_with_only_unknown_features_does_not_apply() {
        // Unknown feature keys are dropped by serde, leaving every known flag unset.
        // Allow-rule is excluded.
        assert!(!allow_features(no_features()));

        // Disallow-rule abstains: it neither denies an allow-rule nor allows by itself.
        let rules = [
            Rule {
                action: RuleAction::Allow,
                os: None,
                features: None,
            },
            feature_rule(RuleAction::Disallow, no_features()),
        ];
        assert!(parse_rules(&rules, "x64", true));
    }

    #[rstest]
    #[case::demo_user_true_is_denied(
        FeatureRule { is_demo_user: Some(true), ..no_features() },
        false
    )]
    #[case::demo_user_false_is_allowed(
        FeatureRule { is_demo_user: Some(false), ..no_features() },
        true
    )]
    #[case::custom_resolution_true_is_allowed(
        FeatureRule { has_custom_resolution: Some(true), ..no_features() },
        true
    )]
    #[case::custom_resolution_false_is_denied(
        FeatureRule { has_custom_resolution: Some(false), ..no_features() },
        false
    )]
    #[case::quick_plays_support_true_is_denied(
        FeatureRule { has_quick_plays_support: Some(true), ..no_features() },
        false
    )]
    #[case::quick_plays_support_false_is_allowed(
        FeatureRule { has_quick_plays_support: Some(false), ..no_features() },
        true
    )]
    #[case::quick_play_singleplayer_true_is_denied(
        FeatureRule { is_quick_play_singleplayer: Some(true), ..no_features() },
        false
    )]
    #[case::quick_play_singleplayer_false_is_allowed(
        FeatureRule { is_quick_play_singleplayer: Some(false), ..no_features() },
        true
    )]
    #[case::quick_play_multiplayer_true_is_denied(
        FeatureRule { is_quick_play_multiplayer: Some(true), ..no_features() },
        false
    )]
    #[case::quick_play_multiplayer_false_is_allowed(
        FeatureRule { is_quick_play_multiplayer: Some(false), ..no_features() },
        true
    )]
    #[case::quick_play_realms_true_is_denied(
        FeatureRule { is_quick_play_realms: Some(true), ..no_features() },
        false
    )]
    #[case::quick_play_realms_false_is_allowed(
        FeatureRule { is_quick_play_realms: Some(false), ..no_features() },
        true
    )]
    // Custom resolution matches, but the demo flag does not
    #[case::all_specified_flags_demo_mismatch_is_denied(
        FeatureRule {
            is_demo_user: Some(true),
            has_custom_resolution: Some(true),
            ..no_features()
        },
        false
    )]
    #[case::all_specified_flags_match_is_allowed(
        FeatureRule {
            is_demo_user: Some(false),
            has_custom_resolution: Some(true),
            ..no_features()
        },
        true
    )]
    #[case::all_specified_flags_third_flag_mismatch_is_denied(
        FeatureRule {
            is_demo_user: Some(false),
            has_custom_resolution: Some(true),
            is_quick_play_realms: Some(true),
            ..no_features()
        },
        false
    )]
    fn allow_feature_rule_requires_all_flags(
        #[case] features: FeatureRule,
        #[case] expected: bool,
    ) {
        assert_eq!(allow_features(features), expected);
    }

    #[rstest]
    // Condition holds (custom resolution) -> disallowed
    #[case::condition_holds_is_denied(
        FeatureRule { has_custom_resolution: Some(true), ..no_features() },
        false
    )]
    // Condition does not hold (demo user wanted) -> allowed
    #[case::condition_does_not_hold_is_allowed(
        FeatureRule { is_demo_user: Some(true), ..no_features() },
        true
    )]
    fn disallow_feature_rule_inverts(#[case] features: FeatureRule, #[case] expected: bool) {
        let rules = [feature_rule(RuleAction::Disallow, features)];
        assert_eq!(parse_rules(&rules, "x64", true), expected);
    }

    #[rstest]
    #[case::matching_os_holding_features(matching(), true, true)]
    #[case::matching_os_failing_features(matching(), false, false)]
    #[case::non_matching_os_holding_features(not_matching(), true, false)]
    #[case::non_matching_os_failing_features(not_matching(), false, false)]
    fn rule_with_os_and_features_needs_both_to_hold(
        #[case] os: OsRule,
        #[case] features_hold: bool,
        #[case] expected: bool,
    ) {
        let features = if features_hold {
            FeatureRule {
                has_custom_resolution: Some(true),
                ..no_features()
            }
        } else {
            FeatureRule {
                is_demo_user: Some(true),
                ..no_features()
            }
        };
        let rules = [Rule {
            action: RuleAction::Allow,
            os: Some(os),
            features: Some(features),
        }];
        assert_eq!(parse_rules(&rules, "x64", true), expected);
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

    fn os_cond(name: Option<Os>, version: Option<&str>, arch: Option<&str>) -> OsRule {
        OsRule {
            name,
            version: version.map(str::to_string),
            arch: arch.map(str::to_string),
        }
    }

    #[rstest]
    #[case::empty_matches_updated(os_cond(None, None, None), "x64", true, true)]
    #[case::empty_matches_not_updated(os_cond(None, None, None), "x64", false, true)]
    #[case::arch_x86_never_matches_updated(os_cond(None, None, Some("x86")), "x64", true, false)]
    #[case::arch_x86_never_matches_not_updated(
        os_cond(None, None, Some("x86")),
        "x64",
        false,
        false
    )]
    #[case::arch_arm_never_matches_updated(os_cond(None, None, Some("arm")), "x64", true, false)]
    #[case::arch_arm_never_matches_not_updated(
        os_cond(None, None, Some("arm")),
        "x64",
        false,
        false
    )]
    #[case::arch_x64_matches_updated(os_cond(None, None, Some("x64")), "x64", true, true)]
    #[case::arch_x64_matches_not_updated(os_cond(None, None, Some("x64")), "x64", false, true)]
    #[case::arch_aarch64_matches_updated(os_cond(None, None, Some("aarch64")), "x64", true, true)]
    #[case::arch_aarch64_matches_not_updated(
        os_cond(None, None, Some("aarch64")),
        "x64",
        false,
        true
    )]
    #[case::arch_is_independent_of_java_arch(
        os_cond(None, None, Some("x86")),
        "aarch64",
        true,
        false
    )]
    #[case::native_name_matches_for_x64_updated(matching(), "x64", true, true)]
    #[case::native_name_matches_for_x64_not_updated(matching(), "x64", false, true)]
    #[case::unknown_name_does_not_match_updated(not_matching(), "x64", true, false)]
    #[case::unknown_name_does_not_match_not_updated(not_matching(), "x64", false, false)]
    // updated: the plain native OS name still matches
    #[case::native_name_aarch64_java_updated(matching(), "aarch64", true, true)]
    // not updated: only the arch-specific variant matches
    #[case::native_name_aarch64_java_not_updated(matching(), "aarch64", false, false)]
    #[case::arch_variant_name_matching_java_arch_updated(
        os_name(Os::native_arch("aarch64")),
        "aarch64",
        true,
        true
    )]
    #[case::arch_variant_name_matching_java_arch_not_updated(
        os_name(Os::native_arch("aarch64")),
        "aarch64",
        false,
        true
    )]
    #[case::arch_variant_name_other_java_arch_updated(
        os_name(Os::native_arch("aarch64")),
        "x64",
        true,
        false
    )]
    #[case::arch_variant_name_other_java_arch_not_updated(
        os_name(Os::native_arch("aarch64")),
        "x64",
        false,
        false
    )]
    #[case::version_regex_matching_anything_keeps_match(
        OsRule { version: Some(".*".to_string()), ..matching() },
        "x64",
        true,
        true
    )]
    #[case::version_regex_never_matching_rejects(
        os_cond(None, Some("a^b"), None),
        "x64",
        true,
        false
    )]
    #[case::invalid_version_regex_is_ignored(os_cond(None, Some("("), None), "x64", true, true)]
    #[case::conditions_are_anded(
        os_cond(Some(Os::native()), Some(".*"), Some("x86")),
        "x64",
        true,
        false
    )]
    fn parse_os_rule_matches(
        #[case] rule: OsRule,
        #[case] java_arch: &str,
        #[case] minecraft_updated: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(parse_os_rule(&rule, java_arch, minecraft_updated), expected);
    }

    #[test]
    fn version_pattern_for_windows_10_matches_java_style_version() {
        assert!(version_matches(r"^10\.", "10.0"));
        assert!(!version_matches(r"^10\.", "6.2"));
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
}
