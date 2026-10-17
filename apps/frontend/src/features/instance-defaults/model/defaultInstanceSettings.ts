import type {
  DefaultInstanceSettingsDto,
  EditDefaultInstanceSettingsDto,
  HooksDto,
  MemorySettingsDto,
  WindowSizeDto,
} from '@/platform/bindings/settings';

export type DefaultInstanceSettings = DefaultInstanceSettingsDto;

export type WindowSize = WindowSizeDto;

export type MemorySettings = MemorySettingsDto;

export type Hooks = HooksDto;

export type EditDefaultInstanceSettings = EditDefaultInstanceSettingsDto;

export const isEditDefaultInstanceSettingsEmpty = (
  dto: EditDefaultInstanceSettings,
) => Object.values(dto).every((value) => value === undefined);
