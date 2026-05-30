import type {
  DefaultInstanceSettingsDto,
  EditDefaultInstanceSettingsDto,
  HooksDto,
  MemorySettingsDto,
  WindowSizeDto,
} from '../api';

export type DefaultInstanceSettings = DefaultInstanceSettingsDto;

export type WindowSize = WindowSizeDto;

export type MemorySettings = MemorySettingsDto;

export type Hooks = HooksDto;

export type EditDefaultInstanceSettings = EditDefaultInstanceSettingsDto;

export const isEditDefaultInstanceSettingsEmpty = (
  dto: EditDefaultInstanceSettings,
) => Object.values(dto).every((value) => value === undefined);
