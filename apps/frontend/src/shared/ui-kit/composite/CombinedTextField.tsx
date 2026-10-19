import type { PolymorphicProps } from '@kobalte/core';
import type { Component, JSX } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import type { PartialBy } from '@/shared/utils';

import { cn } from '@/shared/utils';

import type {
  TextFieldInputProps,
  TextFieldLabelProps,
  TextFieldRootProps,
} from '../primitives/TextField';

import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '../primitives/TextField';

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
    'validationState',
    'class',
  ]);

  const resolvedValidationState = createMemo(
    () => local.validationState ?? (local.errorMessage ? 'invalid' : 'valid'),
  );

  return (
    <TextField
      class={cn('gap-2 flex flex-col', local.class)}
      validationState={resolvedValidationState()}
      {...others}
    >
      <Show when={local.label}>
        <TextFieldLabel {...local.labelProps}>{local.label}</TextFieldLabel>
      </Show>
      <div class='relative flex'>
        <div class='inset-y-0 left-0 absolute'>{local.leadingIcons}</div>
        <TextFieldInput
          type='text'
          {...local.inputProps}
          // The primitive merges this class via cn (tailwind-merge), so pl/pr
          // override its px-3 and the icon padding is not lost.
          class={cn(
            !!local.leadingIcons && 'pl-9',
            !!local.trailingIcons && 'pr-9',
            local.inputProps?.class,
          )}
        />
        <div class='inset-y-0 right-0 absolute'>{local.trailingIcons}</div>
      </div>
      <TextFieldErrorMessage>{local.errorMessage}</TextFieldErrorMessage>
    </TextField>
  );
};
