import {
  isEditDefaultInstanceSettingsEmpty,
  useDefaultInstanceSettings,
  useEditDefaultInstanceSettings,
} from '@/entities/settings';
import { createMemo } from 'solid-js';
import {
  defaultInstanceSettingsToWindowSettingsValues,
  windowSettingsValuesToEditDefaultInstanceSettings,
} from '../model';
import type { WindowSchemaValuesOutput } from '@/widgets/instance-settings-dialog';

export const useWindowSettingsHandler = () => {
  const settings = useDefaultInstanceSettings();
  const editSettings = useEditDefaultInstanceSettings();

  const initialValues = createMemo(() =>
    defaultInstanceSettingsToWindowSettingsValues(settings.data),
  );

  const onChange = (values: Partial<WindowSchemaValuesOutput>) => {
    const dto = windowSettingsValuesToEditDefaultInstanceSettings(values);

    if (isEditDefaultInstanceSettingsEmpty(dto)) {
      return;
    }

    editSettings.mutateAsync(dto);
  };

  return { initialValues, onChange };
};
