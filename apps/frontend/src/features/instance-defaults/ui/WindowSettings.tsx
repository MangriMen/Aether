import type { WindowSettingsFormProps } from '@/features/instance-settings';

import { WindowSettingsForm } from '@/features/instance-settings';

import { useDefaultWindowSettingsHandler } from '../lib/useDefaultWindowSettingsHandler';

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
