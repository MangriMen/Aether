import { Component, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Checkbox,
  CombinedTextField,
  CombinedTextFieldProps,
  LabeledField,
} from '@/shared/ui';

export type CustomTextFieldProps = CombinedTextFieldProps & {
  fieldLabel?: string;
  placeholder?: string;
};

const CustomTextField: Component<CustomTextFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'fieldLabel',
    'placeholder',
    'label',
    'inputProps',
    'class',
  ]);

  const [custom, setCustom] = createSignal(false);

  return (
    <LabeledField class={cn('text-base', local.class)} label={local.fieldLabel}>
      <Checkbox checked={custom()} onChange={setCustom} label={local.label} />
      <CombinedTextField
        disabled={!custom()}
        inputProps={{
          type: 'text',
          placeholder: local.placeholder,
          ...local.inputProps,
        }}
        {...others}
      />
    </LabeledField>
  );
};

export default CustomTextField;
