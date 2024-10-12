/// A version of a Minecraft mod loader
export interface LoaderVersion {
  /// The version ID of the loader
  id: string;
  /// The URL of the version's manifest
  url: string;
  /// Whether the loader is stable or not
  stable: boolean;
}
