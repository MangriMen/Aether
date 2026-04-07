import type { ModLoader } from '@/entities/minecraft/@x/instances';
import type { DefaultInstanceSettings } from '@/entities/settings/@x/instances';

import type { InstanceInstallStage } from './installStage';
import type { LoaderVersionPreference } from './loaderVersionPreference';
import type { PackInfo } from './packInfo';

export interface InstanceSettings extends Partial<DefaultInstanceSettings> {
  forceFullscreen?: boolean;
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
