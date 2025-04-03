export interface Settings {
  launcherDir?: string;
  metadataDir?: string;

  maxConcurrentDownloads: number;

  memory: MemorySettings;

  gameResolution: WindowSize;
  customEnvVars: Array<[string, string]>;
  extraLaunchArgs: string[];

  hooks: Hooks;

  enabledPlugins: Set<string>;
}

export type WindowSize = [number, number];

export interface MemorySettings {
  maximum: number;
}

export interface Hooks {
  pre_launch: string;
  wrapper: string;
  post_exit: string;
}
