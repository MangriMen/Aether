//! Which version of a mod or a pack to install when none is asked for: the same rule for both,
//! and for a first install and an update alike, after Modrinth App's `ReleaseChannel`.

use chrono::{DateTime, Utc};

use super::ContentVersionType;

/// How unstable a version a user takes. The channels nest: `Beta` takes releases too, `Alpha`
/// takes everything.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseChannel {
    #[default]
    Release,
    Beta,
    Alpha,
}

impl ReleaseChannel {
    /// The channel a version of this type was taken from: a user on a beta keeps getting betas.
    pub fn of(version_type: &ContentVersionType) -> Self {
        match version_type {
            ContentVersionType::Release => Self::Release,
            ContentVersionType::Beta => Self::Beta,
            ContentVersionType::Alpha => Self::Alpha,
        }
    }

    #[must_use]
    pub fn least_stable(self, other: Self) -> Self {
        self.max(other)
    }

    /// The version types this channel looks at, widened step by step for a project that has
    /// none of them: a project with only betas still installs, from a release channel.
    pub fn version_type_fallbacks(self) -> &'static [&'static [ContentVersionType]] {
        const RELEASE: &[ContentVersionType] = &[ContentVersionType::Release];
        const BETA: &[ContentVersionType] =
            &[ContentVersionType::Release, ContentVersionType::Beta];
        const ALPHA: &[ContentVersionType] = &[
            ContentVersionType::Release,
            ContentVersionType::Beta,
            ContentVersionType::Alpha,
        ];

        match self {
            Self::Release => &[RELEASE, BETA, ALPHA],
            Self::Beta => &[BETA, ALPHA],
            Self::Alpha => &[ALPHA],
        }
    }
}

/// What [`select_version`] needs to know of a platform's version.
pub trait ChannelVersion {
    /// The platform's id of the version, as a request names the installed one.
    fn id(&self) -> &str;
    fn version_type(&self) -> ContentVersionType;
    /// `None` — the platform's date could not be read; such a version counts as the oldest.
    fn date_published(&self) -> Option<DateTime<Utc>>;
}

impl<V: ChannelVersion> ChannelVersion for &V {
    fn id(&self) -> &str {
        (**self).id()
    }

    fn version_type(&self) -> ContentVersionType {
        (**self).version_type()
    }

    fn date_published(&self) -> Option<DateTime<Utc>> {
        (**self).date_published()
    }
}

/// The version to install out of `versions` (those that fit the target), given the one
/// `installed` now, if any.
///
/// The channel is the least stable of `instance_channel` and the installed version's type. Of
/// the first step of [`ReleaseChannel::version_type_fallbacks`] that any of `versions` belongs
/// to, the newest version is taken — but never one older than the installed one: an update
/// does not roll back, and with nothing newer the installed version stays (it is returned).
///
/// An installed version that is not among `versions` does not fit the target any more (the
/// instance moved to another game version): it cannot stay, so it only sets the channel, and
/// the newest fitting version is taken whatever its date.
///
/// `None` only when there is nothing to take: `versions` is empty and nothing fitting is
/// installed.
pub fn select_version<'a, V: ChannelVersion>(
    versions: &'a [V],
    installed: Option<&'a V>,
    instance_channel: ReleaseChannel,
) -> Option<&'a V> {
    let channel = installed.map_or(instance_channel, |installed| {
        instance_channel.least_stable(ReleaseChannel::of(&installed.version_type()))
    });
    let floor = installed.filter(|installed| {
        versions
            .iter()
            .any(|version| version.id() == installed.id())
    });

    let Some(types) = channel.version_type_fallbacks().iter().find(|types| {
        versions
            .iter()
            .any(|version| types.contains(&version.version_type()))
    }) else {
        return floor;
    };

    versions
        .iter()
        .filter(|version| types.contains(&version.version_type()))
        .filter(|version| {
            floor.is_none_or(|floor| version.date_published() > floor.date_published())
        })
        .max_by_key(ChannelVersion::date_published)
        .or(floor)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Version {
        id: &'static str,
        kind: ContentVersionType,
        day: u32,
    }

    impl ChannelVersion for Version {
        fn id(&self) -> &str {
            self.id
        }

        fn version_type(&self) -> ContentVersionType {
            self.kind.clone()
        }

        fn date_published(&self) -> Option<DateTime<Utc>> {
            format!("2026-10-{:02}T00:00:00Z", self.day).parse().ok()
        }
    }

    fn release(id: &'static str, day: u32) -> Version {
        Version {
            id,
            kind: ContentVersionType::Release,
            day,
        }
    }

    fn beta(id: &'static str, day: u32) -> Version {
        Version {
            id,
            kind: ContentVersionType::Beta,
            day,
        }
    }

    fn alpha(id: &'static str, day: u32) -> Version {
        Version {
            id,
            kind: ContentVersionType::Alpha,
            day,
        }
    }

    fn select<'a>(versions: &'a [Version], installed: Option<&str>) -> Option<&'a str> {
        let installed = installed.map(|id| versions.iter().find(|v| v.id == id).unwrap());
        select_version(versions, installed, ReleaseChannel::Release).map(|v| v.id)
    }

    #[test]
    fn should_install_the_newest_release_when_a_newer_beta_exists() {
        let versions = [release("r1", 1), release("r2", 2), beta("b3", 3)];

        assert_eq!(select(&versions, None), Some("r2"));
    }

    #[test]
    fn should_keep_a_release_when_only_a_beta_is_newer() {
        let versions = [release("r1", 1), beta("b2", 2)];

        assert_eq!(select(&versions, Some("r1")), Some("r1"));
    }

    #[test]
    fn should_move_from_a_beta_when_a_newer_release_exists() {
        let versions = [beta("b1", 1), release("r2", 2)];

        assert_eq!(select(&versions, Some("b1")), Some("r2"));
    }

    #[test]
    fn should_move_from_a_beta_when_a_newer_beta_exists() {
        let versions = [release("r1", 1), beta("b2", 2), beta("b3", 3)];

        assert_eq!(select(&versions, Some("b2")), Some("b3"));
    }

    /// A beta user is not rolled back to the newest release, which is older than their beta.
    #[test]
    fn should_keep_the_installed_version_when_the_newest_release_is_older() {
        let versions = [release("r1", 1), beta("b2", 2)];

        assert_eq!(select(&versions, Some("b2")), Some("b2"));
    }

    #[test]
    fn should_take_an_alpha_when_the_project_has_only_alphas() {
        let versions = [alpha("a1", 1), alpha("a2", 2)];

        assert_eq!(select(&versions, None), Some("a2"));
    }

    #[test]
    fn should_take_a_beta_when_the_project_has_no_releases_but_betas_and_alphas() {
        let versions = [beta("b1", 1), alpha("a2", 2)];

        assert_eq!(select(&versions, None), Some("b1"));
    }

    #[test]
    fn should_follow_the_instance_channel_when_it_is_less_stable() {
        let versions = [release("r1", 1), beta("b2", 2)];

        assert_eq!(
            select_version(&versions, None, ReleaseChannel::Beta).map(|v| v.id),
            Some("b2")
        );
    }

    /// An installed beta that no longer fits the target still sets the channel, but is no floor:
    /// the newest fitting version of the channel is taken even if it is older.
    #[test]
    fn should_replace_an_installed_version_when_it_no_longer_fits_the_target() {
        let fitting = [release("r1", 1), beta("b2", 2)];
        let installed = beta("b9", 9);

        assert_eq!(
            select_version(&fitting, Some(&installed), ReleaseChannel::Release).map(|v| v.id),
            Some("b2")
        );
    }

    #[test]
    fn should_select_nothing_when_there_are_no_versions() {
        assert_eq!(select(&[], None), None);
    }
}
