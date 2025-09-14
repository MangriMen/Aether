export enum ModLoader {
  Fabric = 'fabric',
  Forge = 'forge',
  NeoForge = 'neoforge',
  Quilt = 'quilt',
  Vanilla = 'vanilla',
}

export interface LoaderManifest {
  /// The game versions the mod loader supports
  gameVersions: MinecraftLoaderVersion[];
}

/// A version of a Minecraft mod loader
export interface LoaderVersion {
  /// The version ID of the loader
  id: string;
  /// Whether the loader is stable or not
  stable: boolean;
  /// The URL of the version's manifest
  url: string;
}

export interface MappedLoaderManifest {
  gameVersions: Record<string, MinecraftLoaderVersion>;
}

///  A game version of Minecraft
export interface MinecraftLoaderVersion {
  /// The minecraft version ID
  id: string;
  /// A map that contains loader versions for the game version
  loaders: LoaderVersion[];
  /// Whether the release is stable or not
  stable: boolean;
}
