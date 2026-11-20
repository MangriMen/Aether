/// Classpath entries are separated by `;` on Windows and `:` everywhere else.
///
/// Set by the OS the launcher runs on: the architecture of the Java runtime changes nothing.
pub fn get_classpath_separator() -> &'static str {
    classpath_separator_for(std::env::consts::OS)
}

fn classpath_separator_for(os: &str) -> &'static str {
    if os == "windows" { ";" } else { ":" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::windows("windows", ";")]
    #[case::macos("macos", ":")]
    #[case::linux("linux", ":")]
    #[case::unknown("freebsd", ":")]
    fn should_pick_the_classpath_separator_when_given_the_os(
        #[case] os: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(classpath_separator_for(os), expected);
    }
}
