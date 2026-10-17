import type {
  ActionOnInstanceLaunchDto,
  AppSettingsDto,
  EditAppSettingsDto,
  WindowEffectDto,
} from '@/platform/bindings/settings';

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type AppSettings = AppSettingsDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type EditAppSettings = EditAppSettingsDto;

export type ActionOnInstanceLaunchType = ActionOnInstanceLaunchDto;

export type WindowEffect = WindowEffectDto;
