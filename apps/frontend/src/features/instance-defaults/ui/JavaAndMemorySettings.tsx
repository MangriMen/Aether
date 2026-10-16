import type { JavaAndMemorySettingsFormProps } from '@/features/instance-settings';

import { JavaAndMemorySettingsForm } from '@/features/instance-settings';

import { useDefaultJavaAndMemorySettingsHandler } from '../lib/useDefaultJavaAndMemorySettingsHandler';

export type JavaAndMemorySettingsProps = Omit<
  JavaAndMemorySettingsFormProps,
  'initialValues' | 'onChangePartial'
>;

export const JavaAndMemorySettings = (props: JavaAndMemorySettingsProps) => {
  const { initialValues, onChange } = useDefaultJavaAndMemorySettingsHandler();

  return (
    <JavaAndMemorySettingsForm
      initialValues={initialValues}
      onChangePartial={onChange}
      {...props}
    />
  );
};
