import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { JSX, ValidComponent } from 'solid-js';

import * as ComboboxPrimitive from '@kobalte/core/combobox';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/utils';

const Combobox = ComboboxPrimitive.Root;
const ComboboxHiddenSelect = ComboboxPrimitive.HiddenSelect;

type ComboboxRootProps<
  Option,
  OptGroup,
  T extends ValidComponent = 'div',
> = ComboboxPrimitive.ComboboxRootProps<Option, OptGroup, T> & {
  class?: string | undefined;
  children?: JSX.Element;
};

type ComboboxControlProps<
  Option,
  T extends ValidComponent = 'div',
> = ComboboxPrimitive.ComboboxControlProps<Option, T> & {
  class?: string | undefined;
  children?: JSX.Element;
};

/** The frame of the input and the trigger: looks like a [`SelectTrigger`](./Select.tsx). */
const ComboboxControl = <Option, T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ComboboxControlProps<Option, T>>,
) => {
  const [local, others] = splitProps(props as ComboboxControlProps<Option>, [
    'class',
    'children',
  ]);

  return (
    <ComboboxPrimitive.Control
      class={cn(
        `
          h-9 rounded-md bg-card/card text-sm ring-offset-background
          focus-within:ring-ring
          data-disabled:cursor-not-allowed data-disabled:opacity-50
          data-invalid:border-destructive
          flex w-full items-center border focus-within:ring-2
          enabled:hover:bg-card/hover
        `,
        local.class,
      )}
      {...others}
    >
      {local.children}
    </ComboboxPrimitive.Control>
  );
};

type ComboboxInputProps<T extends ValidComponent = 'input'> =
  ComboboxPrimitive.ComboboxInputProps<T> & {
    class?: string | undefined;
  };

const ComboboxInput = <T extends ValidComponent = 'input'>(
  props: PolymorphicProps<T, ComboboxInputProps<T>>,
) => {
  const [local, others] = splitProps(props as ComboboxInputProps, ['class']);

  return (
    <ComboboxPrimitive.Input
      class={cn(
        `
          h-full min-w-0 bg-transparent px-3 py-2 text-sm
          placeholder:text-muted-foreground
          flex-1 outline-none
          disabled:cursor-not-allowed
        `,
        local.class,
      )}
      {...others}
    />
  );
};

type ComboboxTriggerProps<T extends ValidComponent = 'button'> =
  ComboboxPrimitive.ComboboxTriggerProps<T> & {
    class?: string | undefined;
  };

const ComboboxTrigger = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, ComboboxTriggerProps<T>>,
) => {
  const [local, others] = splitProps(props as ComboboxTriggerProps, ['class']);

  return (
    <ComboboxPrimitive.Trigger
      class={cn('px-2 h-full flex items-center outline-none', local.class)}
      {...others}
    >
      <ComboboxPrimitive.Icon
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
      </ComboboxPrimitive.Icon>
    </ComboboxPrimitive.Trigger>
  );
};

type ComboboxContentProps<T extends ValidComponent = 'div'> =
  ComboboxPrimitive.ComboboxContentProps<T> & { class?: string | undefined };

const ComboboxContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ComboboxContentProps<T>>,
) => {
  const [local, others] = splitProps(props as ComboboxContentProps, ['class']);

  return (
    <ComboboxPrimitive.Portal>
      <ComboboxPrimitive.Content
        // As in `SelectContent`: the focus traps of an enclosing dialog ignore the list.
        data-kb-top-layer=''
        class={cn(
          `
            min-w-32 animate-content-hide rounded-md bg-popover/hard
            text-popover-foreground shadow-lg
            data-expanded:animate-content-show
            relative z-50 origin-(--kb-combobox-content-transform-origin)
            overflow-hidden border
          `,
          local.class,
        )}
        {...others}
      >
        <ComboboxPrimitive.Listbox class='m-0 p-1 max-h-full' />
      </ComboboxPrimitive.Content>
    </ComboboxPrimitive.Portal>
  );
};

type ComboboxItemProps<T extends ValidComponent = 'li'> =
  ComboboxPrimitive.ComboboxItemProps<T> & {
    class?: string | undefined;
    children?: JSX.Element;
  };

const ComboboxItem = <T extends ValidComponent = 'li'>(
  props: PolymorphicProps<T, ComboboxItemProps<T>>,
) => {
  const [local, others] = splitProps(props as ComboboxItemProps, [
    'class',
    'children',
  ]);

  return (
    <ComboboxPrimitive.Item
      class={cn(
        `
          mt-0 rounded-sm py-1.5 pr-8 pl-2 text-sm
          hover:bg-secondary/hover hover:text-foreground
          data-highlighted:bg-secondary/control
          data-highlighted:text-foreground
          relative flex w-full cursor-default items-center outline-none
          select-none
          data-disabled:pointer-events-none data-disabled:opacity-50
        `,
        local.class,
      )}
      {...others}
    >
      <ComboboxPrimitive.ItemIndicator
        class='
          right-2 size-3.5 absolute flex items-center justify-center
        '
      >
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
      </ComboboxPrimitive.ItemIndicator>
      <ComboboxPrimitive.ItemLabel>
        {local.children}
      </ComboboxPrimitive.ItemLabel>
    </ComboboxPrimitive.Item>
  );
};

export type {
  ComboboxRootProps,
  ComboboxControlProps,
  ComboboxInputProps,
  ComboboxTriggerProps,
  ComboboxContentProps,
  ComboboxItemProps,
};

export {
  Combobox,
  ComboboxHiddenSelect,
  ComboboxControl,
  ComboboxInput,
  ComboboxTrigger,
  ComboboxContent,
  ComboboxItem,
};
