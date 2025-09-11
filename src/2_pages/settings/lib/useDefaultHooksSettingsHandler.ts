import type { HooksSettingsSchemaValuesOutput } from '@/entities/settings';
import {
  isEditDefaultInstanceSettingsEmpty,
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '@/entities/settings';
import { createMemo } from 'solid-js';
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
