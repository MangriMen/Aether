import { createMemo } from 'solid-js';

import type { WindowSettingsSchemaOutput } from '@/features/instance-settings/window-settings-form';

import {
  isEditDefaultInstanceSettingsEmpty,
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '@/entities/settings';

import {
  defaultInstanceSettingsToWindowSettingsValues,
  windowSettingsValuesToEditDefaultInstanceSettings,
} from '../model';

export const useDefaultWindowSettingsHandler = () => {
  const settings = useDefaultInstanceSettings();
  const editSettings = useEditDefaultInstanceSettings();

  const initialValues = createMemo(() =>
    defaultInstanceSettingsToWindowSettingsValues(settings.data),
  );

  const onChange = (values: Partial<WindowSettingsSchemaOutput>) => {
    const dto = windowSettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    editSettings.mutateAsync(dto);
  };

  return { initialValues, onChange };
};
