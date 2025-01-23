import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import * as SelectPrimitive from '@kobalte/core/select';
import { createVirtualizer } from '@tanstack/solid-virtual';
import { cva } from 'class-variance-authority';
import type { Component, JSX, ValidComponent } from 'solid-js';
import { createMemo, Show, splitProps, For } from 'solid-js';

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
        'flex h-10 w-full items-center justify-between data-[invalid]:border-destructive rounded-md border border-input bg-secondary px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
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

type VirtualizedOption = Record<string, string | number | bigint>;

type NonVirtualizedProps = {
  virtualized?: false;
  options?: never;
  optionValue?: never;
  itemComponent?: never;
};

type VirtualizedProps<Option> = {
  options: Option[];
  optionValue?: string;
  itemComponent?: Component<
    SelectPrimitive.SelectRootItemComponentProps<Option> & {
      style: JSX.CSSProperties;
    }
  >;
};

type VirtualizedContentProps<Option> = {
  virtualized: true;
} & VirtualizedProps<Option>;

type SelectContentVirtualizedProps<
  Option,
  T extends ValidComponent = 'div',
> = SelectContentProps<T> &
  (NonVirtualizedProps | VirtualizedContentProps<Option>);

const SelectContent = <Option, T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SelectContentVirtualizedProps<Option, T>>,
) => {
  const [local, others] = splitProps(
    props as SelectContentVirtualizedProps<Option, T>,
    ['virtualized', 'options', 'optionValue', 'itemComponent', 'class'],
  );

  return (
    <SelectPrimitive.Portal>
      <SelectPrimitive.Content
        class={cn(
          'relative z-50 min-w-32 overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md origin-[var(--kb-select-content-transform-origin)] animate-content-hide data-[expanded]:animate-content-show',
          local.class,
        )}
        {...(others as SelectContentProps)}
      >
        <Show
          when={local.virtualized && !!local.options}
          fallback={<SelectPrimitive.Listbox class='m-0 p-1' />}
        >
          <SelectListboxVirtualized
            class='m-0 p-1'
            options={local.options ?? []}
            optionValue={local.optionValue}
            itemComponent={local.itemComponent}
          />
        </Show>
      </SelectPrimitive.Content>
    </SelectPrimitive.Portal>
  );
};

type SelectListboxProps<
  Option,
  OptGroup = never,
  T extends ValidComponent = 'ul',
> = SelectPrimitive.SelectListboxProps<Option, OptGroup, T> & {
  class?: string | undefined;
};

type SelectListboxVirtualizedProps<
  Option,
  OptGroup = never,
  T extends ValidComponent = 'ul',
> = SelectListboxProps<Option, OptGroup, T> &
  Exclude<VirtualizedProps<Option>, 'virtualized'>;

const SelectListboxVirtualized = <Option, T extends ValidComponent = 'ul'>(
  props: PolymorphicProps<T, SelectListboxVirtualizedProps<Option, T>>,
) => {
  let listboxRef: HTMLUListElement | undefined;

  const virtualizer = createMemo(() =>
    createVirtualizer({
      count: props.options.length,
      getScrollElement: () => listboxRef ?? null,
      getItemKey: (index: number) =>
        (props.options[index] as VirtualizedOption)[props.optionValue ?? 'id'],
      estimateSize: () => 32,
      // enableSmoothScroll: false,
      overscan: 5,
    }),
  );

  const scrollToItem = (key: string) =>
    virtualizer()?.scrollToIndex(
      props.options.findIndex(
        (option: Option) => (option as VirtualizedOption).value === key,
      ),
    );

  return (
    <SelectPrimitive.Listbox
      class='m-0 overflow-auto p-1'
      ref={listboxRef}
      scrollToItem={scrollToItem}
    >
      {(items) => (
        <div
          style={{
            height: `${virtualizer().getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          <For each={virtualizer().getVirtualItems()}>
            {(virtualRow) => {
              if (typeof virtualRow.key !== 'string') {
                return;
              }

              const item = items().getItem(virtualRow.key);

              if (!item) {
                return;
              }

              const itemStyle: JSX.CSSProperties = {
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start}px)`,
              };

              if (props.itemComponent) {
                return props.itemComponent({ item, style: itemStyle });
              }

              return (
                <SelectPrimitive.Item item={item} style={itemStyle}>
                  <SelectPrimitive.ItemLabel>
                    {item.rawValue.label}
                  </SelectPrimitive.ItemLabel>
                  <SelectPrimitive.ItemIndicator>
                    V
                  </SelectPrimitive.ItemIndicator>
                </SelectPrimitive.Item>
              );
            }}
          </For>
        </div>
      )}
    </SelectPrimitive.Listbox>
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
