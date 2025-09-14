import type { Collection, CollectionNode } from '@kobalte/core';
import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VirtualItem } from '@tanstack/solid-virtual';
import type { Accessor, Component, JSX, ValidComponent } from 'solid-js';

import * as SelectPrimitive from '@kobalte/core/select';
import { createVirtualizer } from '@tanstack/solid-virtual';
import { cva } from 'class-variance-authority';
import { createMemo, For, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

const Select = SelectPrimitive.Root;
const SelectValue = SelectPrimitive.Value;
const SelectHiddenSelect = SelectPrimitive.HiddenSelect;

type SelectRootProps<Option, OptGroup, T extends ValidComponent = 'div'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & SelectPrimitive.SelectRootProps<Option, OptGroup, T>;

type SelectTriggerProps<T extends ValidComponent = 'button'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & SelectPrimitive.SelectTriggerProps<T>;

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
        'flex h-10 w-full items-center justify-between data-[invalid]:border-destructive rounded-md border border-input bg-secondary px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50',
        local.class,
      )}
      {...others}
    >
      {local.children}
      <SelectPrimitive.Icon
        as='svg'
        class='size-4 opacity-50'
        fill='none'
        stroke='currentColor'
        stroke-linecap='round'
        stroke-linejoin='round'
        stroke-width='2'
        viewBox='0 0 24 24'
        xmlns='http://www.w3.org/2000/svg'
      >
        <path d='M8 9l4 -4l4 4' />
        <path d='M16 15l-4 4l-4 -4' />
      </SelectPrimitive.Icon>
    </SelectPrimitive.Trigger>
  );
};

type NonVirtualizedSelectProps = {
  itemComponent?: never;
  options?: never;
  optionValue?: never;
  virtualized?: false;
};

type SelectContentProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & SelectPrimitive.SelectContentProps<T>;

type SelectContentVirtualizedProps<
  Option,
  OptGroup,
  T extends ValidComponent = 'div',
> = (
  | NonVirtualizedSelectProps
  | VirtualizedSelectContentProps<Option, OptGroup>
) &
  SelectContentProps<T>;

type SelectVirtualizedItemComponentProps<Option> = {
  style: JSX.CSSProperties;
} & SelectPrimitive.SelectRootItemComponentProps<Option>;

type VirtualizedSelectContentProps<Option, OptGroup> = {
  virtualized: true;
} & VirtualizedSelectProps<Option, OptGroup>;

type VirtualizedSelectProps<Option, OptGroup> = {
  itemComponent?: Component<SelectVirtualizedItemComponentProps<Option>>;
  options: Array<Option>;
} & Pick<SelectPrimitive.SelectRootOptions<Option, OptGroup>, 'optionValue'>;

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
          {
            'py-1': local.virtualized,
          },
        )}
        {...(others as SelectContentProps)}
      >
        <Show
          fallback={<SelectPrimitive.Listbox class='m-0 max-h-full p-1' />}
          when={local.virtualized && !!local.options}
        >
          <SelectListboxVirtualized
            class='max-h-full px-1'
            itemComponent={local.itemComponent}
            options={local.options ?? []}
            optionValue={local.optionValue}
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
> = {
  class?: string | undefined;
  style?: JSX.CSSProperties;
} & SelectPrimitive.SelectListboxProps<Option, OptGroup, T>;

type SelectListboxVirtualizedProps<
  Option,
  OptGroup = never,
  T extends ValidComponent = 'ul',
> = SelectListboxProps<Option, OptGroup, T> &
  VirtualizedSelectProps<Option, OptGroup>;

const SelectListboxVirtualized = <
  Option,
  OptGroup = never,
  T extends ValidComponent = 'ul',
>(
  props: PolymorphicProps<
    T,
    SelectListboxVirtualizedProps<Option, OptGroup, T>
  >,
) => {
  const [local, _] = splitProps(props, ['optionValue', 'class']);

  let listboxRef: HTMLUListElement | undefined;

  let firstOpen = true;

  const getOptionValue = (option: Option) => {
    const optionValue = local.optionValue ?? null;
    if (optionValue == null) {
      return String(option);
    }
    return String(
      typeof optionValue === 'function'
        ? optionValue(option)
        : option[optionValue],
    );
  };

  const virtualizer = createMemo(() => {
    return createVirtualizer({
      count: props.options.length,
      estimateSize: () => 32,
      getItemKey: (index: number) => getOptionValue(props.options[index]),
      getScrollElement: () => listboxRef ?? null,
      overscan: 5,
    });
  });

  const scrollToItem = (key: string) => {
    const index = props.options.findIndex(
      (option: Option) => getOptionValue(option) === key,
    );

    if (index === 0 && firstOpen) {
      virtualizer()?.scrollToOffset(-100, { align: 'start' });
      firstOpen = false;
      return;
    }

    virtualizer()?.scrollToIndex(index);
  };

  const getItemStyle = (virtualRow: VirtualItem): JSX.CSSProperties => ({
    height: `${virtualRow.size}px`,
    left: 0,
    position: 'absolute',
    top: 0,
    transform: `translateY(${virtualRow.start}px)`,
    width: '100%',
  });

  const renderVirtualItem = (
    items: Accessor<Collection<CollectionNode<Option>>>,
    virtualRow: VirtualItem,
  ) => {
    if (typeof virtualRow.key !== 'string') {
      return;
    }

    const item = items().getItem(virtualRow.key);

    if (!item) {
      return;
    }

    const style = getItemStyle(virtualRow);

    if (props.itemComponent) {
      return <props.itemComponent item={item} style={style} />;
    }

    return (
      <SelectItem item={item} style={style}>
        {getOptionValue(item.rawValue)}
      </SelectItem>
    );
  };

  return (
    <SelectPrimitive.Listbox
      class={cn('overflow-y-auto overflow-x-hidden', local.class)}
      ref={listboxRef}
      scrollToItem={scrollToItem}
    >
      {(items) => (
        <div
          style={{
            height: `${virtualizer().getTotalSize()}px`,
            position: 'relative',
            width: '100%',
          }}
        >
          <For each={virtualizer().getVirtualItems()}>
            {(virtualRow) => renderVirtualItem(items, virtualRow)}
          </For>
        </div>
      )}
    </SelectPrimitive.Listbox>
  );
};

type SelectItemProps<T extends ValidComponent = 'li'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & SelectPrimitive.SelectItemProps<T>;

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
        'relative mt-0 flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        local.class,
      )}
      {...others}
    >
      <SelectPrimitive.ItemIndicator class='absolute right-2 flex size-3.5 items-center justify-center'>
        <svg
          class='size-4'
          fill='none'
          stroke='currentColor'
          stroke-linecap='round'
          stroke-linejoin='round'
          stroke-width='2'
          viewBox='0 0 24 24'
          xmlns='http://www.w3.org/2000/svg'
        >
          <path d='M0 0h24v24H0z' fill='none' stroke='none' />
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

type SelectErrorMessageProps<T extends ValidComponent = 'div'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & SelectPrimitive.SelectErrorMessageProps<T>;

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
  SelectContentProps,
  SelectErrorMessageProps,
  SelectItemProps,
  SelectRootProps,
  SelectTriggerProps,
};

export {
  Select,
  SelectContent,
  SelectErrorMessage,
  SelectHiddenSelect,
  SelectItem,
  SelectTrigger,
  SelectValue,
};
