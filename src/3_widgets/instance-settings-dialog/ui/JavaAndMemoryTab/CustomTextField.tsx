import type { Component } from 'solid-js';
import { createEffect, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import type { CombinedTextFieldProps } from '@/shared/ui';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

export type CustomTextFieldProps = Omit<CombinedTextFieldProps, 'value'> & {
  value?: string | null;
  fieldLabel?: string;
  placeholder?: string;
  onChange?: (value: string | null) => void;
};

const CustomTextField: Component<CustomTextFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'fieldLabel',
    'placeholder',
    'label',
    'onChange',
    'inputProps',
    'class',
  ]);

  const [isCustom, setIsCustom] = createSignal(false);

  createEffect(() => {
    setIsCustom(Boolean(others.defaultValue));
  });

  const [textFieldValue, setTextFieldValue] = createSignal(others.defaultValue);

  const handleChangeIsCustom = (value: boolean) => {
    setIsCustom(value);

    if (value) {
      local.onChange?.(textFieldValue() ?? null);
    } else {
      local.onChange?.(null);
      setTextFieldValue(others.defaultValue);
    }
  };

  const handleChangeTextField = (value: string) => {
    setTextFieldValue(value);
    local.onChange?.(value);
  };

  return (
    <LabeledField class={cn('text-base', local.class)} label={local.fieldLabel}>
      <Checkbox
        label={local.label}
        checked={isCustom()}
        onChange={handleChangeIsCustom}
      />
      <CombinedTextField
        disabled={!isCustom()}
        value={textFieldValue()}
        onChange={setTextFieldValue}
        inputProps={{
          type: 'text',
          placeholder: local.placeholder,
          onBlur: (e) => handleChangeTextField(e.target.value),
          ...local.inputProps,
        }}
        {...others}
      />
    </LabeledField>
  );
};

export default CustomTextField;
