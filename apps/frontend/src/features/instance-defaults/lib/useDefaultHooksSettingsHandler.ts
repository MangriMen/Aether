import { createMemo } from 'solid-js';

import type { HooksSettingsSchemaOutput } from '@/features/instance-settings';

import { logError } from '@/shared/log';

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

    // The mutation already tells the user; this only keeps the rejection handled.
    editSettings.mutateAsync(dto).catch(logError);
  };

  return { initialValues, onChange };
};
