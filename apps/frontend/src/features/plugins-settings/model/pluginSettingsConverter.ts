import type { EditPluginSettings, PluginSettings } from '@/features/plugins';

import type { PluginSettingsSchemaInput } from './pluginSettingsValidation';

export const pluginSettingsToPluginSettingsValues = (
  settings: PluginSettings | null | undefined,
): PluginSettingsSchemaInput | undefined => {
  return {
    allowedHosts: [...(settings?.allowedHosts ?? [])],
    // The folders that were not read are listed with the others, marked with why.
    allowedPaths: [
      ...(settings?.allowedPaths ?? []),
      ...(settings?.ignoredPaths ?? []).map(({ path, reason }) => ({
        ...path,
        ignoredReason: reason,
      })),
    ],
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
    const paths = values.allowedPaths.map(({ ignoredReason, ...path }) => ({
      ignored: ignoredReason !== undefined,
      path: { ...path, readOnly: path.readOnly ?? true },
    }));

    dto.allowedPaths = paths
      .filter(({ ignored }) => !ignored)
      .map(({ path }) => path);
    // The ones the user left in the list stay saved; the others were removed.
    dto.ignoredPaths = paths
      .filter(({ ignored }) => ignored)
      .map(({ path }) => path);
  }

  return dto;
};
