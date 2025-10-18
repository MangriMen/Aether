import type { PluginSettings } from '@/entities/plugins';
import type { PluginSettingsSchemaValues } from '@/features/plugin-settings-form';

export const pluginSettingsToPluginSettingsValues = (
  settings: PluginSettings | undefined,
): PluginSettingsSchemaValues | undefined => {
  return {
    allowedHosts: settings?.allowed_hosts ?? [],
    allowedPaths: settings?.allowed_paths ?? [],
  };
};

export const pluginSettingsValuesToPluginSettings = (
  values: PluginSettingsSchemaValues,
): PluginSettings => ({
  allowed_hosts: values.allowedHosts ?? [],
  allowed_paths: values.allowedPaths ?? [],
});
