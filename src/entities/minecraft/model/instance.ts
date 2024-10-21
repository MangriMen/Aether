export interface Instance {
  installStage: InstanceInstallStage;

  nameId: string;
  path: string;

  name: string;
  iconPath?: string;

  // Main minecraft metadata
  gameVersion: string;
  loader: ModLoader;
  loaderVersion?: string;

  // Runtime metadata
  javaPath?: string;
  extraLaunchArgs?: string[];
  customEnvVars?: { [key: string]: string };

  memory?: MemorySettings;
  forceFullscreen?: boolean;
  gameResolution?: WindowSize;

  // Additional information
  timePlayed: number;
  created: Date;
  modified: Date;
  lastPlayed?: Date;
}

export type WindowSize = [number, number];

export interface MemorySettings {
  maximum: number;
}

export enum InstanceInstallStage {
  /// Instance is installed
  Installed = 'installed',
  /// Instance's minecraft game is still installing
  Installing = 'installing',
  /// Instance created for pack, but the pack hasn't been fully installed yet
  PackInstalling = 'pack_installing',
  /// Instance is not installed
  NotInstalled = 'not_installed',
}

export enum ModLoader {
  Vanilla = 'vanilla',
  Forge = 'forge',
  Fabric = 'fabric',
  Quilt = 'quilt',
  NeoForge = 'neoforge',
}

export interface InstanceCreateDto {
  name: string;
  gameVersion: string;
  modLoader: ModLoader;
  loaderVersion?: string;
  iconPath?: string;
  linkedData?: string;
  skipInstallProfile?: boolean;
}

export interface InstanceLaunchDto {
  instance: Instance;
  envArgs: Array<[string, string]>;
  javaArgs: string[];
  memory: MemorySettings;
  resolution: WindowSize;
  credentials: Credentials;
}

export interface Credentials {
  id: string;
  username: string;
  accessToken: string;
  refreshToken: string;
  expires: Date;
  active: boolean;
}
