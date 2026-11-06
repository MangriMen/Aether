use crate::features::java::JavaDomainError;

/// The version of a Java runtime, as its `java.version` property reports it.
///
/// Both formats are read: the legacy `1.<major>.<minor>_<update>` of Java 8 and older
/// (`1.8.0_292`) and the `<major>.<minor>.<security>` of Java 9 and newer (`17.0.1`, `21`,
/// `21-ea`, `17-internal+0-adhoc.user`). `major` is the feature release (`8`, `17`, `21`): the
/// number runtimes are installed, stored and picked by. Only the major has to be readable;
/// builds spell the rest freely, so a minor that is missing or not a number is `0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JavaVersion {
    major: u32,
    minor: u32,
}

/// The number a version component starts with and the rest of the component.
fn leading_number(part: &str) -> Option<(u32, &str)> {
    let end = part
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(part.len());
    let (digits, rest) = part.split_at(end);
    Some((digits.parse().ok()?, rest))
}

/// The number of a component that must be one: digits, then nothing or a build suffix
/// (`-ea`, `+35`, `_292`).
fn number_component(part: &str) -> Option<u32> {
    let (number, rest) = leading_number(part)?;
    (rest.is_empty() || rest.starts_with(['-', '+', '_'])).then_some(number)
}

impl JavaVersion {
    pub fn parse(version: &str) -> Result<Self, JavaDomainError> {
        let invalid = || JavaDomainError::InvalidVersion {
            version: version.to_owned(),
        };

        let mut parts = version.split('.');
        let first = parts
            .next()
            .and_then(number_component)
            .ok_or_else(invalid)?;

        let major = if first == 1 {
            // Legacy `1.8.0_292`: the feature release is the second component.
            parts
                .next()
                .and_then(number_component)
                .ok_or_else(invalid)?
        } else {
            first
        };
        if major == 0 {
            return Err(invalid());
        }
        let minor = parts
            .next()
            .and_then(leading_number)
            .map_or(0, |(minor, _)| minor);

        Ok(Self { major, minor })
    }

    pub fn major(self) -> u32 {
        self.major
    }

    pub fn minor(self) -> u32 {
        self.minor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::legacy_format("1.8.0_292", 8, 0)]
    #[case::legacy_format_361("1.8.0_361", 8, 0)]
    #[case::major_only_9("9", 9, 0)]
    #[case::major_only_17("17", 17, 0)]
    #[case::major_only_21("21", 21, 0)]
    #[case::detailed_format("17.0.1", 17, 0)]
    #[case::detailed_format_four_parts("17.0.2.3", 17, 0)]
    #[case::build_suffix("11.0.2+9", 11, 0)]
    #[case::minor_release("11.1.2", 11, 1)]
    #[case::early_access("21-ea", 21, 0)]
    #[case::local_build("17-internal+0-adhoc.user", 17, 0)]
    #[case::ci_build("21-internal-adhoc.runner.openjdk", 21, 0)]
    #[case::non_numeric_minor("17.x", 17, 0)]
    fn should_parse_the_major_and_minor_version_when_the_version_is_valid(
        #[case] input: &str,
        #[case] major: u32,
        #[case] minor: u32,
    ) {
        let version = JavaVersion::parse(input).unwrap();

        assert_eq!((version.major(), version.minor()), (major, minor));
    }

    #[rstest]
    #[case::empty("")]
    #[case::not_numeric("abc")]
    #[case::partly_non_numeric("1.abc")]
    #[case::legacy_without_major("1")]
    #[case::zero("0")]
    #[case::trailing_garbage("17abc")]
    fn should_refuse_the_version_when_it_is_invalid(#[case] input: &str) {
        let err = JavaVersion::parse(input).unwrap_err();

        assert!(matches!(err, JavaDomainError::InvalidVersion { .. }));
    }
}
