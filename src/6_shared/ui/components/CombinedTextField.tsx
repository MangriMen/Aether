import type { PolymorphicProps } from '@kobalte/core';
import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type { TextFieldInputProps, TextFieldRootProps } from './TextField';

import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from './TextField';

export type CombinedTextFieldProps = {
  errorMessage?: string;
  inputProps?: PolymorphicProps<'input', TextFieldInputProps<'input'>>;
  label?: string;
} & TextFieldRootProps;

export const CombinedTextField: Component<CombinedTextFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'label',
    'errorMessage',
    'inputProps',
    'class',
  ]);

  return (
    <TextField
      class={cn('flex flex-col gap-2', local.class)}
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      {...others}
    >
      <TextFieldLabel class='flex flex-col gap-2'>
        {local.label}
        <TextFieldInput type='text' {...local.inputProps} />
      </TextFieldLabel>
      <TextFieldErrorMessage>{local.errorMessage}</TextFieldErrorMessage>
    </TextField>
  );
};
