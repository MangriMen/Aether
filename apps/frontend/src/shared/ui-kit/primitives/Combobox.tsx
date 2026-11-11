import type { Collection, CollectionNode } from '@kobalte/core';
import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VirtualItem } from '@tanstack/solid-virtual';
import type { Accessor, Component, JSX, ValidComponent } from 'solid-js';

import * as ComboboxPrimitive from '@kobalte/core/combobox';
import { createVirtualizer } from '@tanstack/solid-virtual';
import { For, Show, splitProps } from 'solid-js';
import { Dynamic } from 'solid-js/web';

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
          not-data-disabled:hover:bg-card/hover
          flex w-full items-center border focus-within:ring-2
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
        class='
          size-4 opacity-50 transition-transform duration-200 ease-out
          data-expanded:-rotate-180
        '
      >
        <path d='M6 9l6 6l6 -6' />
      </ComboboxPrimitive.Icon>
    </ComboboxPrimitive.Trigger>
  );
};

type ComboboxVirtualizedItemComponentProps<Option> =
  ComboboxPrimitive.ComboboxRootItemComponentProps<Option> & {
    style: JSX.CSSProperties;
  };

type ComboboxContentProps<
  Option = unknown,
  T extends ValidComponent = 'div',
> = ComboboxPrimitive.ComboboxContentProps<T> & {
  class?: string | undefined;
  /**
   * Renders only the rows in view, for lists of hundreds. The `Combobox` must be `virtualized`
   * too (that is Kobalte's switch for its focus handling), and the list needs a height of its own
   * (a `max-h-*` on the content), or it would show every row.
   */
  virtualized?: boolean;
  /** The row of a virtualized list; it must put the `style` it is given on the item. */
  itemComponent?: Component<ComboboxVirtualizedItemComponentProps<Option>>;
};

const ESTIMATED_ROW_HEIGHT = 32;

const ComboboxContent = <Option, T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ComboboxContentProps<Option, T>>,
) => {
  const [local, others] = splitProps(props as ComboboxContentProps<Option>, [
    'class',
    'virtualized',
    'itemComponent',
  ]);

  return (
    <ComboboxPrimitive.Portal>
      <ComboboxPrimitive.Content
        // As in `SelectContent`: the focus traps of an enclosing dialog ignore the list.
        data-kb-top-layer=''
        // A press on the list must not take the focus off the input: leaving the field is what
        // applies a typed name, and choosing an option must not do that first.
        onMouseDown={(e: MouseEvent) => e.preventDefault()}
        class={cn(
          `
            min-w-32 animate-content-hide rounded-md bg-popover/hard
            text-popover-foreground shadow-lg
            data-expanded:animate-content-show
            relative z-50 origin-(--kb-combobox-content-transform-origin)
            overflow-hidden border
          `,
          local.class,
          { 'flex flex-col': local.virtualized },
        )}
        {...others}
      >
        <Show
          when={local.virtualized}
          fallback={<ComboboxPrimitive.Listbox class='m-0 p-1 max-h-full' />}
        >
          <ComboboxListboxVirtualized itemComponent={local.itemComponent} />
        </Show>
      </ComboboxPrimitive.Content>
    </ComboboxPrimitive.Portal>
  );
};

type ComboboxListboxVirtualizedProps<Option> = {
  itemComponent?: Component<ComboboxVirtualizedItemComponentProps<Option>>;
};

/**
 * The rows come from the combobox's own collection, so what the typed text filters out is
 * not drawn, and the keys are the ones the combobox itself navigates by.
 */
const ComboboxListboxVirtualized = <Option,>(
  props: ComboboxListboxVirtualizedProps<Option>,
) => {
  // oxlint-disable-next-line no-unassigned-vars
  let listboxRef: HTMLUListElement | undefined;

  let scrollToKey: ((key: string) => void) | undefined;

  const getItemStyle = (virtualRow: VirtualItem): JSX.CSSProperties => ({
    position: 'absolute',
    top: 0,
    left: 0,
    width: '100%',
    height: `${virtualRow.size}px`,
    transform: `translateY(${virtualRow.start}px)`,
  });

  return (
    <ComboboxPrimitive.Listbox
      class='m-0 p-1 min-h-0 overflow-x-hidden overflow-y-auto'
      ref={listboxRef}
      scrollToItem={(key) => scrollToKey?.(key)}
    >
      {(items: Accessor<Collection<CollectionNode<Option>>>) => {
        const keys = () => [...items()].map((node) => node.key);

        const virtualizer = createVirtualizer({
          get count() {
            return keys().length;
          },
          overscan: 5,
          getScrollElement: () => listboxRef ?? null,
          getItemKey: (index: number) => keys()[index] ?? index,
          estimateSize: () => ESTIMATED_ROW_HEIGHT,
        });

        scrollToKey = (key) => {
          const index = keys().indexOf(key);

          if (index !== -1) {
            virtualizer.scrollToIndex(index);
          }
        };

        return (
          <div
            style={{
              height: `${virtualizer.getTotalSize()}px`,
              width: '100%',
              position: 'relative',
            }}
          >
            <For each={virtualizer.getVirtualItems()}>
              {(virtualRow) => {
                const item = () => items().getItem(String(virtualRow.key));

                return (
                  <Show when={item()}>
                    {(node) => (
                      <Show
                        when={props.itemComponent}
                        fallback={
                          <ComboboxItem
                            item={node()}
                            style={getItemStyle(virtualRow)}
                          >
                            {node().textValue}
                          </ComboboxItem>
                        }
                      >
                        {(Row) => (
                          <Dynamic
                            component={Row()}
                            item={node()}
                            style={getItemStyle(virtualRow)}
                          />
                        )}
                      </Show>
                    )}
                  </Show>
                );
              }}
            </For>
          </div>
        );
      }}
    </ComboboxPrimitive.Listbox>
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
