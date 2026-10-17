use crate::features::minecraft::{modded, vanilla};

impl From<daedalus::minecraft::VersionManifest> for vanilla::VersionManifest {
    fn from(value: daedalus::minecraft::VersionManifest) -> Self {
        Self {
            latest: vanilla::LatestVersion {
                release: value.latest.release,
                snapshot: value.latest.snapshot,
            },
            versions: value.versions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<daedalus::minecraft::Version> for vanilla::Version {
    fn from(value: daedalus::minecraft::Version) -> Self {
        Self {
            id: value.id,
            type_: value.type_.into(),
            url: value.url,
            time: value.time,
            release_time: value.release_time,
            sha1: value.sha1,
            compliance_level: value.compliance_level,
            original_sha1: value.original_sha1,
        }
    }
}

impl From<daedalus::minecraft::VersionType> for vanilla::VersionType {
    fn from(value: daedalus::minecraft::VersionType) -> Self {
        match value {
            daedalus::minecraft::VersionType::Release => Self::Release,
            daedalus::minecraft::VersionType::Snapshot => Self::Snapshot,
            daedalus::minecraft::VersionType::OldAlpha => Self::OldAlpha,
            daedalus::minecraft::VersionType::OldBeta => Self::OldBeta,
        }
    }
}

impl From<daedalus::minecraft::VersionInfo> for vanilla::VersionInfo {
    fn from(value: daedalus::minecraft::VersionInfo) -> Self {
        Self {
            arguments: value.arguments.map(|args| {
                args.into_iter()
                    .map(|(k, v)| (k.into(), v.into_iter().map(Into::into).collect()))
                    .collect()
            }),
            asset_index: value.asset_index.into(),
            assets: value.assets,
            downloads: value
                .downloads
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            id: value.id,
            java_version: value.java_version.map(Into::into),
            libraries: value.libraries.into_iter().map(Into::into).collect(),
            logging: value.logging.map(|logging| {
                logging
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect()
            }),
            main_class: value.main_class,
            minecraft_arguments: value.minecraft_arguments,
            minimum_launcher_version: value.minimum_launcher_version,
            release_time: value.release_time,
            time: value.time,
            type_: value.type_.into(),
            data: value
                .data
                .map(|data| data.into_iter().map(|(k, v)| (k, v.into())).collect()),
            processors: value
                .processors
                .map(|procs| procs.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<daedalus::minecraft::ArgumentType> for vanilla::ArgumentType {
    fn from(value: daedalus::minecraft::ArgumentType) -> Self {
        match value {
            daedalus::minecraft::ArgumentType::Game => Self::Game,
            daedalus::minecraft::ArgumentType::Jvm => Self::Jvm,
            daedalus::minecraft::ArgumentType::DefaultUserJvm => Self::DefaultUserJvm,
        }
    }
}

impl From<daedalus::minecraft::Argument> for vanilla::Argument {
    fn from(value: daedalus::minecraft::Argument) -> Self {
        match value {
            daedalus::minecraft::Argument::Normal(s) => Self::Normal(s),
            daedalus::minecraft::Argument::Ruled { rules, value } => Self::Ruled {
                rules: rules.into_iter().map(Into::into).collect(),
                value: value.into(),
            },
        }
    }
}

impl From<daedalus::minecraft::ArgumentValue> for vanilla::ArgumentValue {
    fn from(value: daedalus::minecraft::ArgumentValue) -> Self {
        match value {
            daedalus::minecraft::ArgumentValue::Single(s) => Self::Single(s),
            daedalus::minecraft::ArgumentValue::Many(v) => Self::Many(v),
        }
    }
}

impl From<daedalus::minecraft::Rule> for vanilla::Rule {
    fn from(value: daedalus::minecraft::Rule) -> Self {
        Self {
            action: value.action.into(),
            os: value.os.map(Into::into),
            features: value.features.map(Into::into),
        }
    }
}

impl From<daedalus::minecraft::RuleAction> for vanilla::RuleAction {
    fn from(value: daedalus::minecraft::RuleAction) -> Self {
        match value {
            daedalus::minecraft::RuleAction::Allow => Self::Allow,
            daedalus::minecraft::RuleAction::Disallow => Self::Disallow,
        }
    }
}

impl From<daedalus::minecraft::OsRule> for vanilla::OsRule {
    fn from(value: daedalus::minecraft::OsRule) -> Self {
        Self {
            name: value.name.map(Into::into),
            version: value.version,
            arch: value.arch,
        }
    }
}

impl From<daedalus::minecraft::Os> for vanilla::Os {
    fn from(value: daedalus::minecraft::Os) -> Self {
        match value {
            daedalus::minecraft::Os::Osx => Self::Osx,
            daedalus::minecraft::Os::OsxArm64 => Self::OsxArm64,
            daedalus::minecraft::Os::Windows => Self::Windows,
            daedalus::minecraft::Os::WindowsArm64 => Self::WindowsArm64,
            daedalus::minecraft::Os::Linux => Self::Linux,
            daedalus::minecraft::Os::LinuxArm64 => Self::LinuxArm64,
            daedalus::minecraft::Os::LinuxArm32 => Self::LinuxArm32,
            daedalus::minecraft::Os::Unknown => Self::Unknown,
        }
    }
}

impl From<daedalus::minecraft::FeatureRule> for vanilla::FeatureRule {
    fn from(value: daedalus::minecraft::FeatureRule) -> Self {
        Self {
            is_demo_user: value.is_demo_user,
            has_custom_resolution: value.has_custom_resolution,
            has_quick_plays_support: value.has_quick_plays_support,
            is_quick_play_singleplayer: value.is_quick_play_singleplayer,
            is_quick_play_multiplayer: value.is_quick_play_multiplayer,
            is_quick_play_realms: value.is_quick_play_realms,
        }
    }
}

impl From<daedalus::minecraft::AssetIndex> for vanilla::AssetIndex {
    fn from(value: daedalus::minecraft::AssetIndex) -> Self {
        Self {
            id: value.id,
            sha1: value.sha1,
            size: value.size,
            total_size: value.total_size,
            url: value.url,
        }
    }
}

impl From<daedalus::minecraft::DownloadType> for vanilla::DownloadType {
    fn from(value: daedalus::minecraft::DownloadType) -> Self {
        match value {
            daedalus::minecraft::DownloadType::Client => Self::Client,
            daedalus::minecraft::DownloadType::ClientMappings => Self::ClientMappings,
            daedalus::minecraft::DownloadType::Server => Self::Server,
            daedalus::minecraft::DownloadType::ServerMappings => Self::ServerMappings,
            daedalus::minecraft::DownloadType::WindowsServer => Self::WindowsServer,
        }
    }
}

impl From<daedalus::minecraft::Download> for vanilla::Download {
    fn from(value: daedalus::minecraft::Download) -> Self {
        Self {
            sha1: value.sha1,
            size: value.size,
            url: value.url,
        }
    }
}

impl From<daedalus::minecraft::JavaVersion> for vanilla::JavaVersion {
    fn from(value: daedalus::minecraft::JavaVersion) -> Self {
        Self {
            component: value.component,
            major_version: value.major_version,
        }
    }
}

impl From<daedalus::minecraft::LoggingSide> for vanilla::LoggingSide {
    fn from(value: daedalus::minecraft::LoggingSide) -> Self {
        match value {
            daedalus::minecraft::LoggingSide::Client => Self::Client,
        }
    }
}

impl From<daedalus::minecraft::LoggingConfiguration> for vanilla::LoggingConfiguration {
    fn from(value: daedalus::minecraft::LoggingConfiguration) -> Self {
        match value {
            daedalus::minecraft::LoggingConfiguration::Log4j2Xml { argument, file } => {
                Self::Log4j2Xml {
                    argument,
                    file: file.into(),
                }
            }
        }
    }
}

impl From<daedalus::minecraft::LogConfigDownload> for vanilla::LogConfigDownload {
    fn from(value: daedalus::minecraft::LogConfigDownload) -> Self {
        Self {
            id: value.id,
            sha1: value.sha1,
            size: value.size,
            url: value.url,
        }
    }
}

impl From<daedalus::modded::SidedDataEntry> for modded::SidedDataEntry {
    fn from(value: daedalus::modded::SidedDataEntry) -> Self {
        Self {
            client: value.client,
            server: value.server,
        }
    }
}

impl From<daedalus::modded::Processor> for modded::Processor {
    fn from(value: daedalus::modded::Processor) -> Self {
        Self {
            jar: value.jar,
            classpath: value.classpath,
            args: value.args,
            outputs: value.outputs,
            sides: value.sides,
        }
    }
}

impl From<daedalus::minecraft::Asset> for vanilla::Asset {
    fn from(value: daedalus::minecraft::Asset) -> Self {
        Self {
            hash: value.hash,
            size: value.size,
        }
    }
}

impl From<daedalus::minecraft::AssetsIndex> for vanilla::AssetsIndex {
    fn from(value: daedalus::minecraft::AssetsIndex) -> Self {
        Self {
            objects: value
                .objects
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<daedalus::modded::Manifest> for modded::Manifest {
    fn from(value: daedalus::modded::Manifest) -> Self {
        Self {
            game_versions: value.game_versions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<daedalus::modded::Version> for modded::Version {
    fn from(value: daedalus::modded::Version) -> Self {
        Self {
            id: value.id,
            stable: value.stable,
            loaders: value.loaders.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<daedalus::modded::LoaderVersion> for modded::LoaderVersion {
    fn from(value: daedalus::modded::LoaderVersion) -> Self {
        Self {
            id: value.id,
            url: value.url,
            stable: value.stable,
        }
    }
}

impl From<daedalus::minecraft::Library> for vanilla::Library {
    fn from(value: daedalus::minecraft::Library) -> Self {
        Self {
            downloads: value.downloads.map(Into::into),
            extract: value.extract.map(Into::into),
            name: value.name,
            url: value.url,
            natives: value
                .natives
                .map(|natives| natives.into_iter().map(|(k, v)| (k.into(), v)).collect()),
            rules: value
                .rules
                .map(|rules| rules.into_iter().map(Into::into).collect()),
            checksums: value.checksums,
            include_in_classpath: value.include_in_classpath,
            downloadable: value.downloadable,
        }
    }
}

impl From<daedalus::minecraft::LibraryDownloads> for vanilla::LibraryDownloads {
    fn from(value: daedalus::minecraft::LibraryDownloads) -> Self {
        Self {
            artifact: value.artifact.map(Into::into),
            classifiers: value.classifiers.map(|classifiers| {
                classifiers
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect()
            }),
        }
    }
}

impl From<daedalus::minecraft::LibraryDownload> for vanilla::LibraryDownload {
    fn from(value: daedalus::minecraft::LibraryDownload) -> Self {
        Self {
            path: value.path,
            sha1: value.sha1,
            size: value.size,
            url: value.url,
        }
    }
}

impl From<daedalus::minecraft::LibraryExtract> for vanilla::LibraryExtract {
    fn from(value: daedalus::minecraft::LibraryExtract) -> Self {
        Self {
            exclude: value.exclude,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VANILLA: &str = r#"{
        "id": "1.20.1",
        "type": "release",
        "mainClass": "net.minecraft.client.main.Main",
        "assets": "5",
        "assetIndex": {"id": "5", "sha1": "a", "size": 1, "totalSize": 2, "url": "u"},
        "downloads": {"client": {"sha1": "c", "size": 3, "url": "cu"}},
        "javaVersion": {"component": "java-runtime-gamma", "majorVersion": 17},
        "minimumLauncherVersion": 21,
        "releaseTime": "2023-06-12T00:00:00+00:00",
        "time": "2023-06-12T00:00:00+00:00",
        "arguments": {
            "game": ["--username", {"rules": [{"action": "allow", "features": {"is_demo_user": true}}], "value": "--demo"}],
            "jvm": ["-Xss1M"]
        },
        "libraries": [
            {"name": "org.ow2.asm:asm:9.3", "downloads": {"artifact": {"path": "p", "sha1": "s", "size": 4, "url": "au"}}},
            {"name": "com.mojang:logging:1.1.1", "rules": [{"action": "allow", "os": {"name": "osx", "arch": "x86"}}]}
        ]
    }"#;

    const PARTIAL: &str = r#"{
        "id": "fabric-loader-0.15.0-${modrinth.gameVersion}",
        "inheritsFrom": "${modrinth.gameVersion}",
        "type": "release",
        "mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient",
        "releaseTime": "2023-06-12T00:00:00+00:00",
        "time": "2023-06-12T00:00:00+00:00",
        "arguments": {"jvm": ["-DFabricMcEmu=net.minecraft.client.main.Main"]},
        "libraries": [
            {"name": "org.ow2.asm:asm:9.6", "url": "https://maven.fabricmc.net/"},
            {"name": "net.fabricmc:intermediary:${modrinth.gameVersion}", "url": "https://maven.fabricmc.net/"}
        ]
    }"#;

    #[test]
    fn maps_vanilla_version_info_fields() {
        let info: vanilla::VersionInfo =
            serde_json::from_str::<daedalus::minecraft::VersionInfo>(VANILLA)
                .unwrap()
                .into();

        assert_eq!(info.main_class, "net.minecraft.client.main.Main");
        assert_eq!(info.asset_index.total_size, 2);
        assert_eq!(info.java_version.unwrap().major_version, 17);
        assert_eq!(info.libraries.len(), 2);
        assert!(
            info.libraries[0]
                .downloads
                .as_ref()
                .unwrap()
                .artifact
                .is_some()
        );
        assert_eq!(
            info.libraries[1].rules.as_ref().unwrap()[0]
                .os
                .as_ref()
                .unwrap()
                .arch
                .as_deref(),
            Some("x86")
        );
        let game = &info.arguments.unwrap()[&vanilla::ArgumentType::Game];
        assert!(matches!(game[0], vanilla::Argument::Normal(_)));
        assert!(matches!(game[1], vanilla::Argument::Ruled { .. }));
    }

    #[test]
    fn merging_loader_profile_keeps_vanilla_and_loader_data() {
        let merged = daedalus::modded::merge_partial_version(
            serde_json::from_str(PARTIAL).unwrap(),
            serde_json::from_str(VANILLA).unwrap(),
        );
        let info: vanilla::VersionInfo = merged.into();

        assert_eq!(info.id, "fabric-loader-0.15.0-1.20.1");
        assert_eq!(
            info.main_class,
            "net.fabricmc.loader.impl.launch.knot.KnotClient"
        );
        assert_eq!(info.asset_index.id, "5");

        let names: Vec<_> = info.libraries.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "com.mojang:logging:1.1.1",
                "org.ow2.asm:asm:9.6",
                "net.fabricmc:intermediary:1.20.1",
            ]
        );
        // The loader's asm supersedes the vanilla one.
        let jvm = &info.arguments.unwrap()[&vanilla::ArgumentType::Jvm];
        assert_eq!(jvm.len(), 2);
    }
}
