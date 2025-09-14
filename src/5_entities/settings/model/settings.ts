export interface Hooks {
  post_exit: string;
  pre_launch: string;
  wrapper: string;
}

export interface MemorySettings {
  maximum: number;
}

export interface Settings {
  customEnvVars: Array<[string, string]>;
  enabledPlugins: Set<string>;

  extraLaunchArgs: string[];

  gameResolution: WindowSize;

  hooks: Hooks;
  launcherDir?: string;
  maxConcurrentDownloads: number;

  memory: MemorySettings;

  metadataDir?: string;
}

export type WindowSize = [number, number];
