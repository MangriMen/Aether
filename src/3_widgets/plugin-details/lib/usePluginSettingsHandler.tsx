import type { SubmitHandler } from '@modular-forms/solid';

import { createMemo, type Accessor } from 'solid-js';

import type {
  EditPluginSettings,
  PluginMetadata,
  PluginSettings,
} from '@/entities/plugins';
import type { PluginSettingsSchemaInput } from '@/features/plugin-settings-form';

import {
  pluginSettingsToPluginSettingsValues,
  pluginSettingsValuesToPluginSettings,
} from '../model';

export interface UsePluginSettingsHandler {
  pluginId: Accessor<PluginMetadata['id']>;
  pluginSettings: Accessor<PluginSettings | undefined>;
  editPluginSettings: Accessor<
    (args: {
      id: PluginMetadata['id'];
      editSettings: EditPluginSettings;
    }) => void
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

  const onSubmit: SubmitHandler<PluginSettingsSchemaInput> = (values) => {
    editPluginSettings()({
      id: pluginId(),
      editSettings: pluginSettingsValuesToPluginSettings(values),
    });
  };

  return { initialValues, onSubmit };
};
