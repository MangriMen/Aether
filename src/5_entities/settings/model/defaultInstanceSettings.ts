export interface DefaultInstanceSettings {
  launchArgs: string[];
  envVars: Array<[string, string]>;

  memory: MemorySettings;
  gameResolution: WindowSize;

  hooks: Hooks;
}

export type WindowSize = [number, number];

export interface MemorySettings {
  maximum: number;
}

export interface Hooks {
  preLaunch: string | null;
  wrapper: string | null;
  postExit: string | null;
}

export type EditDefaultInstanceSettings = Partial<
  Omit<DefaultInstanceSettings, 'hooks'>
> & {
  hooks?: Partial<Hooks>;
};

export const isEditDefaultInstanceSettingsEmpty = (
  dto: EditDefaultInstanceSettings,
) => Object.values(dto).every((value) => value === undefined);
