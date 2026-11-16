import { createMemo } from 'solid-js';

import type { WindowSettingsSchemaOutput } from '@/features/instance-settings';

import {
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '../api/defaultInstanceSettingsQueries';
import { isEditDefaultInstanceSettingsEmpty } from '../model/defaultInstanceSettings';
import {
  defaultInstanceSettingsToWindowSettingsValues,
  windowSettingsValuesToEditDefaultInstanceSettings,
} from '../model/defaultInstanceSettingsConverter';

export const useDefaultWindowSettingsHandler = () => {
  const settings = useDefaultInstanceSettings();
  const editSettings = useEditDefaultInstanceSettings();

  const initialValues = createMemo(() =>
    defaultInstanceSettingsToWindowSettingsValues(settings.data),
  );

  // The form handles a failed save: it logs it and puts the stored values back.
  const onChange = async (values: Partial<WindowSettingsSchemaOutput>) => {
    if (!settings.data) {
      return;
    }

    const dto = windowSettingsValuesToEditDefaultInstanceSettings(
      settings.data,
      values,
    );

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    await editSettings.mutateAsync(dto);
  };

  return { initialValues, onChange };
};
