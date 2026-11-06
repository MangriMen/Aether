import type {
  AllowedPathRejectionDto,
  EditPluginSettingsDto,
  PluginSettingsDto,
} from '@/platform/bindings/plugin';

export type PluginSettings = PluginSettingsDto;

export type EditPluginSettings = EditPluginSettingsDto;

/** Why a saved folder was not given to the plugin. */
export type AllowedPathRejection = AllowedPathRejectionDto;

export const ALLOWED_PATH_REJECTIONS = [
  'host_not_absolute',
  'host_not_a_folder',
  'host_is_root',
  'plugin_not_absolute',
  'plugin_not_normalized',
  'plugin_is_root',
  'plugin_reserved',
  'host_overlaps_core',
] as const satisfies readonly AllowedPathRejection[];
