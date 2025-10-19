import type { EditPluginSettings, PluginSettings } from '@/entities/plugins';
import type { PluginSettingsSchemaInput } from '@/features/plugin-settings-form';

export const pluginSettingsToPluginSettingsValues = (
  settings: PluginSettings | undefined,
): PluginSettingsSchemaInput | undefined => {
  return {
    allowedHosts: [...(settings?.allowedHosts ?? [])],
    allowedPaths: [...(settings?.allowedPaths ?? [])],
  };
};

export const pluginSettingsValuesToEditPluginSettings = (
  values: PluginSettingsSchemaInput,
): EditPluginSettings => {
  const dto: EditPluginSettings = {};

  if (values.allowedHosts !== undefined) {
    dto.allowedHosts = [...values.allowedHosts];
  }

  if (values.allowedPaths !== undefined) {
    dto.allowedPaths = [...values.allowedPaths];
  }

  return dto;
};
