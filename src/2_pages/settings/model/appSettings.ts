export type ActionOnInstanceLaunchType = 'hide' | 'close' | 'nothing';
export type MicaMode = 'off' | 'light' | 'dark' | 'system';

export interface AppSettings {
  actionOnInstanceLaunch: ActionOnInstanceLaunchType;
  mica: MicaMode;
  transparent: boolean;
}

export type UpdateAppSettings = Partial<AppSettings>;
