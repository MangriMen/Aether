import type { PolymorphicProps } from '@kobalte/core';
import type { Component, JSX } from 'solid-js';
import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type {
  TextFieldInputProps,
  TextFieldLabelProps,
  TextFieldRootProps,
} from './TextField';
import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from './TextField';

export type CombinedTextFieldProps = TextFieldRootProps & {
  label?: JSX.Element;
  errorMessage?: string;
  labelProps?: TextFieldLabelProps<'label'>;
  inputProps?: PolymorphicProps<'input', TextFieldInputProps<'input'>>;
};

export const CombinedTextField: Component<CombinedTextFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'label',
    'errorMessage',
    'labelProps',
    'inputProps',
    'class',
  ]);

  return (
    <TextField
      class={cn('flex flex-col gap-2', local.class)}
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      {...others}
    >
      <Show when={local.label}>
        <TextFieldLabel {...local.labelProps}>{local.label}</TextFieldLabel>
      </Show>
      <TextFieldInput type='text' {...local.inputProps} />
      <TextFieldErrorMessage>{local.errorMessage}</TextFieldErrorMessage>
    </TextField>
  );
};
