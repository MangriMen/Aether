import type { ModLoader } from '@/entities/minecraft/@x/instances';
import type { Hooks, MemorySettings, WindowSize } from '@/entities/settings';

import type { LoaderVersionPreference } from './loaderVersionPreference';
import type { PackInfo } from './packInfo';

export interface NewInstance {
  name: string;
  gameVersion: string;
  modLoader: ModLoader;
  loaderVersion?: LoaderVersionPreference;
  iconPath?: string;
  skipInstallProfile?: boolean;
  packInfo?: PackInfo;
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

export interface EditInstance extends EditInstanceSettings {
  name?: string;
  javaPath?: string | null;
}

export interface ImportInstance {
  pluginId: string;
  importerId: string;
  path: string;
}

export const isEditInstanceSettingsEmpty = (dto: EditInstanceSettings) =>
  Object.values(dto).every((value) => value === undefined);

export const isEditInstanceEmpty = (dto: EditInstance) =>
  isEditInstanceSettingsEmpty(dto) && dto.name === undefined;
