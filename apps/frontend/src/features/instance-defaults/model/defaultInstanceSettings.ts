import type {
  DefaultInstanceSettingsDto,
  EditDefaultInstanceSettingsDto,
  HooksDto,
  MemorySettingsDto,
  WindowSizeDto,
} from '@/platform/bindings/settings';

export type DefaultInstanceSettings = DefaultInstanceSettingsDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type WindowSize = WindowSizeDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type MemorySettings = MemorySettingsDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type Hooks = HooksDto;

export type EditDefaultInstanceSettings = EditDefaultInstanceSettingsDto;

export const isEditDefaultInstanceSettingsEmpty = (
  dto: EditDefaultInstanceSettings,
) => Object.values(dto).every((value) => value === undefined);
