import { createMemo, type Component } from 'solid-js';
import {
  type HooksSettingsFormProps,
  type HooksSettingsSchemaValuesOutput,
  HooksSettingsForm,
  useEditGlobalInstanceSettings,
  useGlobalInstanceSettings,
  type EditGlobalInstanceSettings,
} from '@/entities/settings';

import { instanceSettingsToHooksSettingsValues } from '../../model';

export type HooksSettingsProps = Omit<
  HooksSettingsFormProps,
  'initialValues' | 'onChangePartial'
>;

export const HooksSettings: Component<HooksSettingsProps> = (props) => {
  const settings = useGlobalInstanceSettings();
  const editSettings = useEditGlobalInstanceSettings();

  const hooksSettingsInitialValues = createMemo(() =>
    settings.data
      ? instanceSettingsToHooksSettingsValues(settings.data)
      : undefined,
  );

  const onHooksSettingsValuesChange = (
    values: Partial<HooksSettingsSchemaValuesOutput>,
  ) => {
    const dto: EditGlobalInstanceSettings = {};

    if (values.pre_launch && values.wrapper && values.post_exit) {
      dto.hooks = {
        pre_launch: values.pre_launch,
        wrapper: values.wrapper,
        post_exit: values.post_exit,
      };
    }

    editSettings.mutateAsync(dto);
  };

  return (
    <HooksSettingsForm
      initialValues={hooksSettingsInitialValues}
      onChangePartial={onHooksSettingsValuesChange}
      {...props}
    />
  );
};
