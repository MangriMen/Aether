import type { PluginSettings } from '@/entities/plugins';
import type { PluginSettingsSchemaInput } from '@/features/plugin-settings-form';

export const pluginSettingsToPluginSettingsValues = (
  settings: PluginSettings | undefined,
): PluginSettingsSchemaInput | undefined => {
  return {
    allowedHosts: [...(settings?.allowedHosts ?? [])],
    allowedPaths: [...(settings?.allowedPaths ?? [])],
  };
};

export const pluginSettingsValuesToPluginSettings = (
  values: PluginSettingsSchemaInput,
): PluginSettings => ({
  allowedHosts: [...(values.allowedHosts ?? [])],
  allowedPaths: [...(values.allowedPaths ?? [])],
});
