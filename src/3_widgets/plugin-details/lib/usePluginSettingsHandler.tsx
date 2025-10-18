import type { SubmitHandler } from '@modular-forms/solid';

import { createMemo, type Accessor } from 'solid-js';

import type { PluginMetadata, PluginSettings } from '@/entities/plugins';
import type { PluginSettingsSchemaValues } from '@/features/plugin-settings-form';

import {
  pluginSettingsToPluginSettingsValues,
  pluginSettingsValuesToPluginSettings,
} from '../model';

export interface UsePluginSettingsHandler {
  pluginId: Accessor<PluginMetadata['id']>;
  pluginSettings: Accessor<PluginSettings | undefined>;
  editPluginSettings: Accessor<
    (args: { id: PluginMetadata['id']; settings: PluginSettings }) => void
  >;
}

export const usePluginSettingsHandler = ({
  pluginId,
  pluginSettings,
  editPluginSettings,
}: UsePluginSettingsHandler) => {
  const initialValues = createMemo(() =>
    pluginSettingsToPluginSettingsValues(pluginSettings()),
  );

  const onSubmit: SubmitHandler<PluginSettingsSchemaValues> = (values) => {
    editPluginSettings()({
      id: pluginId(),
      settings: pluginSettingsValuesToPluginSettings(values),
    });
  };

  return { initialValues, onSubmit };
};
