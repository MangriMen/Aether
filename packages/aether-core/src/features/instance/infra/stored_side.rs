use tracing::warn;

use crate::features::instance::FileSide;

/// The side a pack file is stored with, or `None` — with a warning — when it is not one (a
/// hand-edited or damaged pack): nothing reads the side back to decide, so a bad one costs the
/// value, not the file.
pub(super) fn parse_stored_side(file_name: &str, side: Option<&str>) -> Option<FileSide> {
    match side?.parse() {
        Ok(side) => Some(side),
        Err(err) => {
            warn!("Ignoring the side of pack file \"{file_name}\": {err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::both("both", FileSide::Both)]
    #[case::client("client", FileSide::Client)]
    #[case::server("server", FileSide::Server)]
    fn should_parse_the_side_when_it_is_one_of_packwizs(
        #[case] stored: &str,
        #[case] expected: FileSide,
    ) {
        assert_eq!(parse_stored_side("a.jar", Some(stored)), Some(expected));
    }

    #[rstest]
    #[case::unknown("everywhere")]
    #[case::wrong_case("Client")]
    #[case::empty("")]
    fn should_drop_the_side_when_it_is_not_one_of_packwizs(#[case] stored: &str) {
        assert_eq!(parse_stored_side("a.jar", Some(stored)), None);
    }

    #[test]
    fn should_keep_no_side_when_none_is_stored() {
        assert_eq!(parse_stored_side("a.jar", None), None);
    }
}
