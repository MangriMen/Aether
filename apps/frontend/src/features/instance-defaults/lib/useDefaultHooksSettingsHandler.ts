import { createMemo } from 'solid-js';

import type { HooksSettingsSchemaOutput } from '@/features/instance-settings';

import {
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '../api/defaultInstanceSettingsQueries';
import { isEditDefaultInstanceSettingsEmpty } from '../model/defaultInstanceSettings';
import {
  defaultInstanceSettingsToHooksSettingsValues,
  hooksSettingsValuesToEditDefaultInstanceSettings,
} from '../model/defaultInstanceSettingsConverter';

export const useDefaultHooksSettingsHandler = () => {
  const settings = useDefaultInstanceSettings();
  const editSettings = useEditDefaultInstanceSettings();

  const initialValues = createMemo(() =>
    defaultInstanceSettingsToHooksSettingsValues(settings.data),
  );

  // The form handles a failed save: it logs it and puts the stored values back.
  const onChange = async (values: Partial<HooksSettingsSchemaOutput>) => {
    const dto = hooksSettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    await editSettings.mutateAsync(dto);
  };

  return { initialValues, onChange };
};
