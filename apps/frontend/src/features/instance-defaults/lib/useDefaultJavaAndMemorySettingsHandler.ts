import { createMemo } from 'solid-js';

import type { JavaAndMemorySettingsSchemaOutput } from '@/features/instance-settings';

import {
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '../api/defaultInstanceSettingsQueries';
import { isEditDefaultInstanceSettingsEmpty } from '../model/defaultInstanceSettings';
import {
  defaultInstanceSettingsToJavaAndMemorySettingsValues,
  javaAndMemorySettingsValuesToEditDefaultInstanceSettings,
} from '../model/defaultInstanceSettingsConverter';

export const useDefaultJavaAndMemorySettingsHandler = () => {
  const settings = useDefaultInstanceSettings();
  const editSettings = useEditDefaultInstanceSettings();

  const initialValues = createMemo(() =>
    defaultInstanceSettingsToJavaAndMemorySettingsValues(settings.data),
  );

  // The form handles a failed save: it logs it and puts the stored values back.
  const onChange = async (
    values: Partial<JavaAndMemorySettingsSchemaOutput>,
  ) => {
    const dto =
      javaAndMemorySettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    await editSettings.mutateAsync(dto);
  };

  return { initialValues, onChange };
};
