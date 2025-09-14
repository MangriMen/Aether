import type { ModLoader } from '@/entities/minecraft/@x/instances';
import type {
  Hooks,
  MemorySettings,
  WindowSize,
} from '@/entities/settings/@x/instances';

import type { ContentType } from './contentType';

export enum InstanceInstallStage {
  /// Instance is installed
  Installed = 'installed',
  /// Instance's minecraft game is still installing
  Installing = 'installing',
  /// Instance is not installed
  NotInstalled = 'not_installed',
  /// Instance created for pack, but the pack hasn't been fully installed yet
  PackInstalling = 'pack_installing',
}

export interface EditInstance {
  customEnvVars?: Array<[string, string]> | null;
  extraLaunchArgs?: null | string[];
  gameResolution?: null | WindowSize;
  javaPath?: null | string;
  memory?: MemorySettings | null;
  name?: string;
}

export interface ImportHandler {
  fieldLabel: string;
  fileExtensions: string[];
  fileName: string;
  packType: string;
  title: string;
}

export interface Instance {
  // Additional information
  created: Date;

  customEnvVars?: Array<[string, string]>;
  extraLaunchArgs?: string[];

  forceFullscreen?: boolean;

  gameResolution?: WindowSize;
  // Main minecraft metadata
  gameVersion: string;
  hooks: Hooks;

  iconPath?: string;
  id: string;
  installStage: InstanceInstallStage;

  // Launch arguments
  javaPath?: string;
  lastPlayed?: string;
  loader: ModLoader;

  loaderVersion?: LoaderVersionPreference;
  // Minecraft runtime settings
  memory?: MemorySettings;
  modified: Date;

  name: string;
  packInfo?: PackInfo;

  recent_time_played: number;

  timePlayed: number;
}

export interface InstanceFile {
  contentType: ContentType;
  disabled: boolean;
  fileName: string;
  hash: string;
  name?: string;
  path: string;
  size: number;
  update: Record<string, Record<string, unknown>>;
}

export interface InstanceImportDto {
  packType: string;
  path: string;
}

export interface InstancePluginSettings {
  preLaunch?: string;
}

export type LoaderVersionPreference = 'latest' | 'stable' | string;

export interface NewInstance {
  gameVersion: string;
  iconPath?: string;
  loaderVersion?: LoaderVersionPreference;
  modLoader: ModLoader;
  name: string;
  packInfo?: PackInfo;
  skipInstallProfile?: boolean;
}

export interface PackInfo {
  canUpdate: boolean;
  packType: string;
  version: string;
}
