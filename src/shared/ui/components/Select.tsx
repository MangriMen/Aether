import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import * as SelectPrimitive from '@kobalte/core/select';
import { cva } from 'class-variance-authority';
import type { JSX, ValidComponent } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

const Select = SelectPrimitive.Root;
const SelectValue = SelectPrimitive.Value;
const SelectHiddenSelect = SelectPrimitive.HiddenSelect;

type SelectRootProps<
  Option,
  OptGroup,
  T extends ValidComponent = 'div',
> = SelectPrimitive.SelectRootProps<Option, OptGroup, T> & {
  class?: string | undefined;
  children?: JSX.Element;
};

type SelectTriggerProps<T extends ValidComponent = 'button'> =
  SelectPrimitive.SelectTriggerProps<T> & {
    class?: string | undefined;
    children?: JSX.Element;
  };

const SelectTrigger = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, SelectTriggerProps<T>>,
) => {
  const [local, others] = splitProps(props as SelectTriggerProps, [
    'class',
    'children',
  ]);
  return (
    <SelectPrimitive.Trigger
      class={cn(
        'flex h-10 w-full items-center justify-between data-[invalid]:border-destructive rounded-md border border-input bg-transparent px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
        local.class,
      )}
      {...others}
    >
      {local.children}
      <SelectPrimitive.Icon
        as='svg'
        xmlns='http://www.w3.org/2000/svg'
        viewBox='0 0 24 24'
        fill='none'
        stroke='currentColor'
        stroke-width='2'
        stroke-linecap='round'
        stroke-linejoin='round'
        class='size-4 opacity-50'
      >
        <path d='M8 9l4 -4l4 4' />
        <path d='M16 15l-4 4l-4 -4' />
      </SelectPrimitive.Icon>
    </SelectPrimitive.Trigger>
  );
};

type SelectContentProps<T extends ValidComponent = 'div'> =
  SelectPrimitive.SelectContentProps<T> & { class?: string | undefined };

const SelectContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SelectContentProps<T>>,
) => {
  const [local, others] = splitProps(props as SelectContentProps, ['class']);
  return (
    <SelectPrimitive.Portal>
      <SelectPrimitive.Content
        class={cn(
          'relative z-50 min-w-32 overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md origin-[var(--kb-select-content-transform-origin)] animate-content-hide data-[expanded]:animate-content-show',
          local.class,
        )}
        {...others}
      >
        <SelectPrimitive.Listbox class='m-0 p-1' />
      </SelectPrimitive.Content>
    </SelectPrimitive.Portal>
  );
};

type SelectItemProps<T extends ValidComponent = 'li'> =
  SelectPrimitive.SelectItemProps<T> & {
    class?: string | undefined;
    children?: JSX.Element;
  };

const SelectItem = <T extends ValidComponent = 'li'>(
  props: PolymorphicProps<T, SelectItemProps<T>>,
) => {
  const [local, others] = splitProps(props as SelectItemProps, [
    'class',
    'children',
  ]);
  return (
    <SelectPrimitive.Item
      class={cn(
        'relative mt-0 flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        local.class,
      )}
      {...others}
    >
      <SelectPrimitive.ItemIndicator class='absolute right-2 flex size-3.5 items-center justify-center'>
        <svg
          xmlns='http://www.w3.org/2000/svg'
          viewBox='0 0 24 24'
          fill='none'
          stroke='currentColor'
          stroke-width='2'
          stroke-linecap='round'
          stroke-linejoin='round'
          class='size-4'
        >
          <path stroke='none' d='M0 0h24v24H0z' fill='none' />
          <path d='M5 12l5 5l10 -10' />
        </svg>
      </SelectPrimitive.ItemIndicator>
      <SelectPrimitive.ItemLabel>{local.children}</SelectPrimitive.ItemLabel>
    </SelectPrimitive.Item>
  );
};

const labelVariants = cva(
  'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
  {
    variants: {
      variant: {
        label: 'data-[invalid]:text-destructive',
        description: 'font-normal text-muted-foreground',
        error: 'text-destructive',
      },
    },
    defaultVariants: {
      variant: 'label',
    },
  },
);

type SelectErrorMessageProps<T extends ValidComponent = 'div'> =
  SelectPrimitive.SelectErrorMessageProps<T> & {
    class?: string | undefined;
    children?: JSX.Element;
  };

const SelectErrorMessage = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SelectErrorMessageProps<T>>,
) => {
  const [local, others] = splitProps(props as SelectErrorMessageProps, [
    'class',
  ]);

  return (
    <SelectPrimitive.ErrorMessage
      class={cn(labelVariants({ variant: 'error' }), local.class)}
      {...others}
    />
  );
};

export type {
  SelectRootProps,
  SelectTriggerProps,
  SelectContentProps,
  SelectItemProps,
  SelectErrorMessageProps,
};

export {
  Select,
  SelectValue,
  SelectHiddenSelect,
  SelectTrigger,
  SelectContent,
  SelectItem,
  SelectErrorMessage,
};
