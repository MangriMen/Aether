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

  const onChange = (values: Partial<HooksSettingsSchemaOutput>) => {
    const dto = hooksSettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    editSettings.mutateAsync(dto);
  };

  return { initialValues, onChange };
};
