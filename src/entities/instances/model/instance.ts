import type { ModLoader } from '@/entities/minecrafts/@x/instances';
import type { ContentType } from './contentType';
import type {
  Hooks,
  MemorySettings,
  WindowSize,
} from '@/entities/settings/@x/instances';

export interface InstancePluginSettings {
  preLaunch?: string;
}

export interface PackInfo {
  packType: string;
  version: string;
  canUpdate: boolean;
}

export interface Instance {
  id: string;
  path: string;

  name: string;
  iconPath?: string;

  installStage: InstanceInstallStage;

  // Main minecraft metadata
  gameVersion: string;
  loader: ModLoader;
  loaderVersion?: string;

  // Launch arguments
  javaPath?: string;
  extraLaunchArgs?: string[];
  customEnvVars?: Array<[string, string]>;

  // Minecraft runtime settings
  memory?: MemorySettings;
  forceFullscreen?: boolean;
  gameResolution?: WindowSize;

  // Additional information
  created: Date;
  modified: Date;
  lastPlayed?: string;

  timePlayed: number;
  recent_time_played: number;

  hooks: Hooks;

  packInfo?: PackInfo;
}

export interface InstanceFile {
  hash: string;
  name?: string;
  fileName: string;
  size: number;
  contentType: ContentType;
  path: string;
  disabled: boolean;
  update: Record<string, Record<string, unknown>>;
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

export interface NewInstance {
  name: string;
  gameVersion: string;
  modLoader: ModLoader;
  loaderVersion?: string;
  iconPath?: string;
  skipInstallProfile?: boolean;
  packInfo?: PackInfo;
}

export interface EditInstance {
  name?: string;
  javaPath?: string | null;
  extraLaunchArgs?: string[] | null;
  customEnvVars?: Array<[string, string]> | null;
  memory?: MemorySettings | null;
  gameResolution?: WindowSize | null;
}

export interface InstanceImportDto {
  packType: string;
  path: string;
}

export interface ImportHandler {
  packType: string;
  title: string;
  fieldLabel: string;
  fileName: string;
  fileExtensions: string[];
}
