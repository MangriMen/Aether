import { createMemo } from 'solid-js';

import type { JavaAndMemorySettingsSchemaOutput } from '@/features/instance-settings/java-and-memory-settings-form';

import {
  isEditDefaultInstanceSettingsEmpty,
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '@/entities/settings';

import {
  defaultInstanceSettingsToJavaAndMemorySettingsValues,
  javaAndMemorySettingsValuesToEditDefaultInstanceSettings,
} from '../model';

export const useJavaAndMemorySettingsHandler = () => {
  const settings = useDefaultInstanceSettings();
  const editSettings = useEditDefaultInstanceSettings();

  const initialValues = createMemo(() =>
    defaultInstanceSettingsToJavaAndMemorySettingsValues(settings.data),
  );

  const onChange = (values: Partial<JavaAndMemorySettingsSchemaOutput>) => {
    const dto =
      javaAndMemorySettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    editSettings.mutateAsync(dto);
  };
  return { initialValues, onChange };
};
