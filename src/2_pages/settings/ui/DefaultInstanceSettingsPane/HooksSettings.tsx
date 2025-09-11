import { type Component } from 'solid-js';
import {
  type HooksSettingsFormProps,
  HooksSettingsForm,
} from '@/entities/settings';

import { useDefaultHooksSettingsHandler } from '../../lib';

export type HooksSettingsProps = Omit<
  HooksSettingsFormProps,
  'initialValues' | 'onChangePartial'
>;

export const HooksSettings: Component<HooksSettingsProps> = (props) => {
  const { initialValues, onChange } = useDefaultHooksSettingsHandler();

  return (
    <HooksSettingsForm
      initialValues={initialValues}
      onChangePartial={onChange}
      {...props}
    />
  );
};
