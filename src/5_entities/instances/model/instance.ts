import type { ModLoader } from '@/entities/minecraft/@x/instances';
import type {
  DefaultInstanceSettings,
  Hooks,
  MemorySettings,
  WindowSize,
} from '@/entities/settings/@x/instances';

import type { ContentType } from './contentType';

export interface InstancePluginSettings {
  preLaunch?: string;
}

export interface PackInfo {
  packType: string;
  version: string;
  canUpdate: boolean;
}

export interface InstanceSettings extends Partial<DefaultInstanceSettings> {
  forceFullscreen?: boolean;
}

export interface EditInstanceSettings {
  // Game settings
  forceFullscreen?: boolean;
  gameResolution?: WindowSize | null;

  // Java launch settings
  memory?: MemorySettings | null;
  launchArgs?: string[] | null;
  envVars?: Array<[string, string]> | null;

  hooks?: Partial<Hooks>;
}

export interface Instance extends InstanceSettings {
  id: string;

  name: string;
  iconPath?: string;

  installStage: InstanceInstallStage;

  // Main minecraft metadata
  gameVersion: string;
  loader: ModLoader;
  loaderVersion?: LoaderVersionPreference;

  javaPath?: string | null;

  // Additional information
  created: Date;
  modified: Date;
  lastPlayed?: string;

  timePlayed: number;
  recent_time_played: number;

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

export type LoaderVersionPreference = 'stable' | 'latest' | string;

export interface NewInstance {
  name: string;
  gameVersion: string;
  modLoader: ModLoader;
  loaderVersion?: LoaderVersionPreference;
  iconPath?: string;
  skipInstallProfile?: boolean;
  packInfo?: PackInfo;
}

export interface EditInstance extends EditInstanceSettings {
  name?: string;
  javaPath?: string | null;
}

export interface ImportInstance {
  pathOrUrl: string;
  pluginId: string | undefined;
}

export interface ImportConfig {
  pluginId: string;
  title: string;
  fieldLabel: string;
  fileName: string;
  fileExtensions: string[];
}

export const isEditInstanceSettingsEmpty = (dto: EditInstanceSettings) =>
  Object.values(dto).every((value) => value === undefined);

export const isEditInstanceEmpty = (dto: EditInstance) =>
  isEditInstanceSettingsEmpty(dto) && dto.name === undefined;
