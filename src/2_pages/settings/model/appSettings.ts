export type ActionOnInstanceLaunchType = 'close' | 'hide' | 'nothing';
export interface AppSettings {
  actionOnInstanceLaunch: ActionOnInstanceLaunchType;
  transparent: boolean;
  windowEffect: WindowEffect;
}

export type UpdateAppSettings = Partial<AppSettings>;

export type WindowEffect =
  | 'acrylic'
  | 'mica_dark'
  | 'mica_light'
  | 'mica'
  | 'off';
