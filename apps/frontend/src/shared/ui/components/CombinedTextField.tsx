import type { PolymorphicProps } from '@kobalte/core';
import type { Component, JSX } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import type { PartialBy } from '@/shared/model';

import type {
  TextFieldInputProps,
  TextFieldLabelProps,
  TextFieldRootProps,
} from '../uikit/TextField';

import { cn } from '../../lib';
import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '../uikit/TextField';

export type CombinedTextFieldProps = TextFieldRootProps & {
  label?: JSX.Element;
  errorMessage?: string;
  labelProps?: TextFieldLabelProps<'label'>;
  inputProps?: PartialBy<
    PolymorphicProps<'input', TextFieldInputProps<'input'>>,
    'type'
  >;
  leadingIcons?: JSX.Element;
  trailingIcons?: JSX.Element;
};

export const CombinedTextField: Component<CombinedTextFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'label',
    'errorMessage',
    'labelProps',
    'inputProps',
    'leadingIcons',
    'trailingIcons',
    'class',
  ]);

  return (
    <TextField
      class={cn('gap-2 flex flex-col', local.class)}
      validationState={local.errorMessage ? 'invalid' : 'valid'}
      {...others}
    >
      <Show when={local.label}>
        <TextFieldLabel {...local.labelProps}>{local.label}</TextFieldLabel>
      </Show>
      <div class='relative flex'>
        <div class='inset-y-0 left-0 absolute'>{local.leadingIcons}</div>
        <TextFieldInput type='text' {...local.inputProps} />
        <div class='inset-y-0 right-0 absolute'>{local.trailingIcons}</div>
      </div>
      <TextFieldErrorMessage>{local.errorMessage}</TextFieldErrorMessage>
    </TextField>
  );
};
