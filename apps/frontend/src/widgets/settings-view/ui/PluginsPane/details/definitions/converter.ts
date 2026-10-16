import type { EditPluginSettings, PluginSettings } from '@/entities/plugins';

import type { PluginSettingsSchemaInput } from '../../settings-form';

export const pluginSettingsToPluginSettingsValues = (
  settings: PluginSettings | null | undefined,
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
    // A folder is read-only unless the user said otherwise.
    dto.allowedPaths = values.allowedPaths.map((path) => ({
      ...path,
      readOnly: path.readOnly ?? true,
    }));
  }

  return dto;
};
