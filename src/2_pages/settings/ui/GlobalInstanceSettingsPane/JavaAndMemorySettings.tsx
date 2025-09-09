import { createMemo } from 'solid-js';

import {
  useEditGlobalInstanceSettings,
  useGlobalInstanceSettings,
} from '@/entities/settings';
import type { JavaAndMemorySettingsSchemaValuesOutput } from '@/widgets/instance-settings-dialog';
import type { EditGlobalInstanceSettings } from '@/entities/settings/model/globalInstanceSettings';
import {
  JavaAndMemorySettingsForm,
  type JavaAndMemorySettingsFormProps,
} from './JavaAndMemorySettingsForm';
import { instanceSettingsToJavaAndMemorySettingsValues } from '../../model';
import {
  stringToEnvVars,
  stringToExtraLaunchArgs,
} from '@/widgets/instance-settings-dialog/lib';

export type JavaAndMemorySettingsProps = Omit<
  JavaAndMemorySettingsFormProps,
  'initialValues' | 'onChangePartial'
>;

export const JavaAndMemorySettings = (props: JavaAndMemorySettingsProps) => {
  const settings = useGlobalInstanceSettings();
  const editSettings = useEditGlobalInstanceSettings();

  const javaAndMemorySettingsInitialValues = createMemo(() =>
    settings.data
      ? instanceSettingsToJavaAndMemorySettingsValues(settings.data)
      : undefined,
  );

  const onJavaAndMemorySettingsValuesChange = (
    values: Partial<JavaAndMemorySettingsSchemaValuesOutput>,
  ) => {
    const dto: EditGlobalInstanceSettings = {};

    if (values.memory?.maximum) {
      dto.memory = { maximum: values.memory.maximum };
    }

    if (values.extraLaunchArgs) {
      dto.extraLaunchArgs = stringToExtraLaunchArgs(values.extraLaunchArgs);
    }

    if (values.customEnvVars) {
      dto.customEnvVars = stringToEnvVars(values.customEnvVars);
    }

    editSettings.mutateAsync(dto);
  };

  return (
    <JavaAndMemorySettingsForm
      initialValues={javaAndMemorySettingsInitialValues}
      onChangePartial={onJavaAndMemorySettingsValuesChange}
      {...props}
    />
  );
};
