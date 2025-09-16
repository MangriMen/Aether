import { createMemo } from 'solid-js';

import type { HooksSettingsSchemaValuesOutput } from '@/features/instance-settings/hooks-settings-form';

import {
  isEditDefaultInstanceSettingsEmpty,
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '@/entities/settings';

import {
  defaultInstanceSettingsToHooksSettingsValues,
  hooksSettingsValuesToEditDefaultInstanceSettings,
} from '../model';

export const useDefaultHooksSettingsHandler = () => {
  const settings = useDefaultInstanceSettings();
  const editSettings = useEditDefaultInstanceSettings();

  const initialValues = createMemo(() =>
    defaultInstanceSettingsToHooksSettingsValues(settings.data),
  );

  const onChange = (values: Partial<HooksSettingsSchemaValuesOutput>) => {
    const dto = hooksSettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    editSettings.mutateAsync(dto);
  };

  return { initialValues, onChange };
};
