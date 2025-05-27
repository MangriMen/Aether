export type ActionOnInstanceLaunchType = 'hide' | 'close' | 'nothing';

export interface AppSettings {
  actionOnInstanceLaunch: ActionOnInstanceLaunchType;
  mica: boolean;
  transparent: boolean;
}

export type UpdateAppSettings = Partial<AppSettings>;
