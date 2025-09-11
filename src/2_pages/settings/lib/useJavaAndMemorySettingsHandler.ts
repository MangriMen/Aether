import {
  isEditDefaultInstanceSettingsEmpty,
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '@/entities/settings';
import { createMemo } from 'solid-js';
import type { JavaAndMemorySettingsSchemaValuesOutput } from '@/widgets/instance-settings-dialog';
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

  const onChange = (
    values: Partial<JavaAndMemorySettingsSchemaValuesOutput>,
  ) => {
    const dto =
      javaAndMemorySettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    editSettings.mutateAsync(dto);
  };
  return { initialValues, onChange };
};
