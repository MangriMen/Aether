import { createMemo, type Accessor } from 'solid-js';

import type {
  EditPluginSettings,
  PluginMetadata,
  PluginSettings,
} from '@/entities/plugins';
import type {
  PluginSettingsSchemaInput,
  PluginSettingsSchemaOutput,
} from '@/features/plugin-settings-form';

import {
  pluginSettingsToPluginSettingsValues,
  pluginSettingsValuesToEditPluginSettings,
} from '../model';

export interface UsePluginSettingsHandler {
  pluginId: Accessor<PluginMetadata['id']>;
  pluginSettings: Accessor<PluginSettings | null | undefined>;
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

  const onChangePartial = (values: PluginSettingsSchemaInput) => {
    editPluginSettings()({
      id: pluginId(),
      editSettings: pluginSettingsValuesToEditPluginSettings(values),
    });
  };

  const onSubmit = (values: PluginSettingsSchemaOutput) =>
    onChangePartial(values);

  return { initialValues, onChangePartial, onSubmit };
};
