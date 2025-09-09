export interface GlobalInstanceSettings {
  extraLaunchArgs: string[];
  customEnvVars: Array<[string, string]>;

  memory: MemorySettings;
  gameResolution: WindowSize;

  hooks: Hooks;
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

export type EditGlobalInstanceSettings = Partial<GlobalInstanceSettings>;
