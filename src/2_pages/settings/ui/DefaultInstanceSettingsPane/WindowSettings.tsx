import {
  WindowSettingsForm,
  type WindowSettingsFormProps,
} from '../../../../4_features/instance-settings/window-settings-form/ui/WindowSettingsForm';
import { useWindowSettingsHandler } from '../../lib';

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
