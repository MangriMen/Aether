import { createMemo } from 'solid-js';

import type { JavaAndMemorySettingsSchemaOutput } from '@/features/instance-settings';

import { logError } from '@/shared/log';

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

  const onChange = (values: Partial<JavaAndMemorySettingsSchemaOutput>) => {
    const dto =
      javaAndMemorySettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    editSettings.mutateAsync(dto).catch(logError);
  };

  return { initialValues, onChange };
};
