export interface AppSettings {
  actionOnInstanceLaunch: ActionOnInstanceLaunchType;
  transparent: boolean;
  windowEffect: WindowEffect;
}

export type EditAppSettings = Partial<AppSettings>;

export type ActionOnInstanceLaunchType = 'hide' | 'close' | 'nothing';

export type WindowEffect =
  | 'off'
  | 'mica_light'
  | 'mica_dark'
  | 'mica'
  | 'acrylic';
