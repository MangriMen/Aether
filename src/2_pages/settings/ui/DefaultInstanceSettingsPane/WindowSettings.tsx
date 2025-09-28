import type { WindowSettingsFormProps } from '@/features/instance-settings/window-settings-form';

import { WindowSettingsForm } from '@/features/instance-settings/window-settings-form';

import { useDefaultWindowSettingsHandler } from '../../lib';

export type WindowSettingsProps = Omit<
  WindowSettingsFormProps,
  'initialValues' | 'onChangePartial'
>;

export const WindowSettings = (props: WindowSettingsProps) => {
  const { initialValues, onChange } = useDefaultWindowSettingsHandler();

  return (
    <WindowSettingsForm
      initialValues={initialValues}
      onChangePartial={onChange}
      {...props}
    />
  );
};
