use regex::Regex;

use crate::features::minecraft::vanilla;

pub fn parse_rules(rules: &[vanilla::Rule], java_version: &str, minecraft_updated: bool) -> bool {
    let mut parse_results = rules
        .iter()
        .map(|rule| parse_rule(rule, java_version, minecraft_updated))
        .collect::<Vec<Option<bool>>>();

    if rules
        .iter()
        .all(|rule| matches!(rule.action, vanilla::RuleAction::Disallow))
    {
        parse_results.push(Some(true))
    }

    !(parse_results.iter().any(|x| x == &Some(false)) || parse_results.iter().all(|x| x.is_none()))
}

fn parse_rule(rule: &vanilla::Rule, java_version: &str, minecraft_updated: bool) -> Option<bool> {
    use vanilla::{Rule, RuleAction};

    let res = match rule {
        Rule { os: Some(os), .. } => parse_os_rule(os, java_version, minecraft_updated),
        Rule {
            features: Some(features),
            ..
        } => {
            !features.is_demo_user.unwrap_or(true)
                || features.has_custom_resolution.unwrap_or(false)
                || !features.has_quick_plays_support.unwrap_or(true)
                || !features.is_quick_play_multiplayer.unwrap_or(true)
                || !features.is_quick_play_realms.unwrap_or(true)
                || !features.is_quick_play_singleplayer.unwrap_or(true)
        }
        _ => return Some(true),
    };

    match rule.action {
        RuleAction::Allow => Some(res),
        RuleAction::Disallow if res => Some(false),
        RuleAction::Disallow => None,
    }
}

// Platform rule resolving
pub fn parse_os_rule(
    rule: &vanilla::OsRule,
    java_arch: &str,
    // Minecraft updated over 1.18.2 (supports MacOS Natively)
    minecraft_updated: bool,
) -> bool {
    use vanilla::Os;

    let mut rule_match = true;

    // Check architecture
    if let Some(ref arch) = rule.arch {
        rule_match &= !matches!(arch.as_str(), "x86" | "arm");
    }

    // Check OS name
    if let Some(name) = &rule.name {
        // If Minecraft is updated and the OS name is not "LinuxArm64" or "LinuxArm32",
        // the rule matches if the native OS or the OS corresponding to the Java architecture matches the OS name
        if minecraft_updated && (name != &Os::LinuxArm64 || name != &Os::LinuxArm32) {
            rule_match &= &Os::native() == name || &Os::native_arch(java_arch) == name;
        } else {
            // If Minecraft is not updated, the rule matches if the OS corresponding to the Java architecture matches the OS name
            rule_match &= &Os::native_arch(java_arch) == name;
        }
    }

    // Check OS version
    if let Some(version) = &rule.version
        && let Ok(regex) = Regex::new(version.as_str())
    {
        rule_match &= regex.is_match(&sys_info::os_release().unwrap_or_default());
    }

    rule_match
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
