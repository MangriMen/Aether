/// The version type
export enum VersionType {
  /// A major version, which is stable for all players to use
  Release = 'release',
  /// An experimental version, which is unstable and used for feature previews and beta testing
  Snapshot = 'snapshot',
  /// The oldest versions before the game was released
  OldAlpha = 'old_alpha',
  /// Early versions of the game
  OldBeta = 'old_beta',
}

/// A game version of Minecraft
export interface Version {
  /// A unique identifier of the version
  id: string;
  /// The release type of the version
  type: VersionType;
  /// A link to additional information about the version
  url: string;
  /// The latest time a file in this version was updated
  time: string;
  /// The time this version was released
  release_time: string;
  /// The SHA1 hash of the additional information about the version
  sha1: string;
  /// Whether the version supports the latest player safety features
  compliance_level: number;
  /// The SHA1 hash of the original unmodified Minecraft versions JSON
  original_sha1?: string;
}

/// The latest snapshot and release of the game
export interface LatestVersion {
  /// The version id of the latest release
  release: string;
  /// The version id of the latest snapshot
  snapshot: string;
}

/// Data of all game versions of Minecraft
export interface VersionManifest {
  /// A struct containing the latest snapshot and release of the game
  latest: LatestVersion;
  /// A list of game versions of Minecraft
  versions: Version[];
}
