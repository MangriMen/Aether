import { useJavaAndMemorySettingsHandler } from '../../lib';
import {
  JavaAndMemorySettingsForm,
  type JavaAndMemorySettingsFormProps,
} from './JavaAndMemorySettingsForm';

export type JavaAndMemorySettingsProps = Omit<
  JavaAndMemorySettingsFormProps,
  'initialValues' | 'onChangePartial'
>;

export const JavaAndMemorySettings = (props: JavaAndMemorySettingsProps) => {
  const { initialValues, onChange } = useJavaAndMemorySettingsHandler();

  return (
    <JavaAndMemorySettingsForm
      initialValues={initialValues}
      onChangePartial={onChange}
      {...props}
    />
  );
};
