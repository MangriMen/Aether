import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import * as TextFieldPrimitive from '@kobalte/core/text-field';
import { cva } from 'class-variance-authority';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

type TextFieldRootProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & TextFieldPrimitive.TextFieldRootProps<T>;

const TextField = TextFieldPrimitive.Root;

type TextFieldInputProps<T extends ValidComponent = 'input'> = {
  class?: string | undefined;
  type:
    | 'button'
    | 'checkbox'
    | 'color'
    | 'date'
    | 'datetime-local'
    | 'email'
    | 'file'
    | 'hidden'
    | 'image'
    | 'month'
    | 'number'
    | 'password'
    | 'radio'
    | 'range'
    | 'reset'
    | 'search'
    | 'submit'
    | 'tel'
    | 'text'
    | 'time'
    | 'url'
    | 'week';
} & TextFieldPrimitive.TextFieldInputProps<T>;

const TextFieldInput = <T extends ValidComponent = 'input'>(
  props: PolymorphicProps<T, TextFieldInputProps<T>>,
) => {
  const [local, others] = splitProps(props as TextFieldInputProps, [
    'type',
    'class',
  ]);
  return (
    <TextFieldPrimitive.Input
      class={cn(
        'flex h-10 w-full rounded-md border border-input data-[invalid]:border-destructive bg-secondary px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50',
        local.class,
      )}
      type={local.type}
      {...others}
    />
  );
};

type TextFieldTextAreaProps<T extends ValidComponent = 'textarea'> = {
  class?: string | undefined;
} & TextFieldPrimitive.TextFieldTextAreaProps<T>;

const TextFieldTextArea = <T extends ValidComponent = 'textarea'>(
  props: PolymorphicProps<T, TextFieldTextAreaProps<T>>,
) => {
  const [local, others] = splitProps(props as TextFieldTextAreaProps, [
    'class',
  ]);
  return (
    <TextFieldPrimitive.TextArea
      class={cn(
        'flex  min-h-[80px] w-full rounded-md border border-input data-[invalid]:border-destructive bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50',
        local.class,
      )}
      {...others}
    />
  );
};

const labelVariants = cva(
  'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
  {
    defaultVariants: {
      variant: 'label',
    },
    variants: {
      variant: {
        description: 'font-normal text-muted-foreground',
        error: 'text-destructive',
        label: 'data-[invalid]:text-destructive',
      },
    },
  },
);

type TextFieldLabelProps<T extends ValidComponent = 'label'> = {
  class?: string | undefined;
} & TextFieldPrimitive.TextFieldLabelProps<T>;

const TextFieldLabel = <T extends ValidComponent = 'label'>(
  props: PolymorphicProps<T, TextFieldLabelProps<T>>,
) => {
  const [local, others] = splitProps(props as TextFieldLabelProps, ['class']);
  return (
    <TextFieldPrimitive.Label
      class={cn(labelVariants(), local.class)}
      {...others}
    />
  );
};

type TextFieldDescriptionProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & TextFieldPrimitive.TextFieldDescriptionProps<T>;

const TextFieldDescription = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TextFieldDescriptionProps<T>>,
) => {
  const [local, others] = splitProps(props as TextFieldDescriptionProps, [
    'class',
  ]);
  return (
    <TextFieldPrimitive.Description
      class={cn(labelVariants({ variant: 'description' }), local.class)}
      {...others}
    />
  );
};

type TextFieldErrorMessageProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & TextFieldPrimitive.TextFieldErrorMessageProps<T>;

const TextFieldErrorMessage = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TextFieldErrorMessageProps<T>>,
) => {
  const [local, others] = splitProps(props as TextFieldErrorMessageProps, [
    'class',
  ]);
  return (
    <TextFieldPrimitive.ErrorMessage
      class={cn(labelVariants({ variant: 'error' }), local.class)}
      {...others}
    />
  );
};

export {
  TextField,
  TextFieldDescription,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
  TextFieldTextArea,
};

export type {
  TextFieldDescriptionProps,
  TextFieldErrorMessageProps,
  TextFieldInputProps,
  TextFieldLabelProps,
  TextFieldRootProps,
  TextFieldTextAreaProps,
};
