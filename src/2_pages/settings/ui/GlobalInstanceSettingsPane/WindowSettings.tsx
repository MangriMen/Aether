import { createMemo } from 'solid-js';
import {
  WindowSettingsForm,
  type WindowSettingsFormProps,
} from './WindowSettingsForm';
import {
  useEditGlobalInstanceSettings,
  useGlobalInstanceSettings,
} from '@/entities/settings';
import { instanceSettingsToWindowSettingsValues } from '../../model';
import type { WindowSchemaValuesOutput } from '@/widgets/instance-settings-dialog';
import type { EditGlobalInstanceSettings } from '@/entities/settings/model/globalInstanceSettings';

export type WindowSettingsProps = Omit<
  WindowSettingsFormProps,
  'initialValues' | 'onChangePartial'
>;

export const WindowSettings = (props: WindowSettingsProps) => {
  const settings = useGlobalInstanceSettings();
  const editSettings = useEditGlobalInstanceSettings();

  const windowSettingsInitialValues = createMemo(() =>
    settings.data
      ? instanceSettingsToWindowSettingsValues(settings.data)
      : undefined,
  );

  const onWindowSettingsValuesChange = (
    values: Partial<WindowSchemaValuesOutput>,
  ) => {
    const dto: EditGlobalInstanceSettings = {};

    if (values.resolution) {
      dto.gameResolution = [values.resolution.width, values.resolution.height];
    }

    editSettings.mutateAsync(dto);
  };

  return (
    <WindowSettingsForm
      initialValues={windowSettingsInitialValues}
      onChangePartial={onWindowSettingsValuesChange}
      {...props}
    />
  );
};
