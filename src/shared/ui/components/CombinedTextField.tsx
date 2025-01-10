import { PolymorphicProps } from '@kobalte/core';
import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldInputProps,
  TextFieldLabel,
  TextFieldRootProps,
} from './TextField';

export type CTextFieldProps = TextFieldRootProps & {
  label?: string;
  errorMessage?: string;
  inputProps?: PolymorphicProps<'input', TextFieldInputProps<'input'>>;
};

export const CombinedTextField: Component<CTextFieldProps> = (props) => {
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
