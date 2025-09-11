import { useWindowSettingsHandler } from '../../lib';
import {
  WindowSettingsForm,
  type WindowSettingsFormProps,
} from './WindowSettingsForm';

export type WindowSettingsProps = Omit<
  WindowSettingsFormProps,
  'initialValues' | 'onChangePartial'
>;

export const WindowSettings = (props: WindowSettingsProps) => {
  const { initialValues, onChange } = useWindowSettingsHandler();

  return (
    <WindowSettingsForm
      initialValues={initialValues}
      onChangePartial={onChange}
      {...props}
    />
  );
};
