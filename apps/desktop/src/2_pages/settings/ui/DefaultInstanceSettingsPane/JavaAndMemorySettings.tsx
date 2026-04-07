import type { JavaAndMemorySettingsFormProps } from '@/features/instance-settings/java-and-memory-settings-form';

import { JavaAndMemorySettingsForm } from '@/features/instance-settings/java-and-memory-settings-form';

import { useDefaultJavaAndMemorySettingsHandler } from '../../lib';

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
