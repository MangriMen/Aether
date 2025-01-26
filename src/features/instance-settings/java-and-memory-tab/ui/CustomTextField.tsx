import type { Component } from 'solid-js';
import { createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import type { CombinedTextFieldProps } from '@/shared/ui';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

export type CustomTextFieldProps = CombinedTextFieldProps & {
  fieldLabel?: string;
  placeholder?: string;
  onChange?: (value: string | null) => void;
};

const CustomTextField: Component<CustomTextFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'fieldLabel',
    'placeholder',
    'label',
    'onChange',
    'inputProps',
    'class',
  ]);

  const [custom, setCustom] = createSignal(!!others.defaultValue);
  const [value, setValue] = createSignal(others.defaultValue);

  const handleSetCustom = (val: boolean) => {
    setCustom(val);
    if (!val) {
      local.onChange?.(null);
    } else {
      local.onChange?.(value() ?? null);
    }
  };

  const handleChange = (value: string) => {
    setValue(value);
    local.onChange?.(value);
  };

  return (
    <LabeledField class={cn('text-base', local.class)} label={local.fieldLabel}>
      <Checkbox
        checked={custom()}
        onChange={handleSetCustom}
        label={local.label}
      />
      <CombinedTextField
        disabled={!custom()}
        value={value()}
        inputProps={{
          type: 'text',
          placeholder: local.placeholder,
          onBlur: (e) => handleChange(e.target.value),
          ...local.inputProps,
        }}
        {...others}
      />
    </LabeledField>
  );
};

export default CustomTextField;
