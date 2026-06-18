import type { Component, ComponentProps } from 'solid-js';

import { For, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type { AllowedItemProps } from '../model';

export type FixedItemsListProps<TValue, TError> = ComponentProps<'ul'> & {
  item: Component<AllowedItemProps<TValue, TError>>;
  items?: TValue[];
};

export const FixedItemsList = <TValue, TError>(
  props: FixedItemsListProps<TValue, TError>,
) => {
  const [local, others] = splitProps(props, ['items', 'item', 'class']);

  return (
    <ul
      class={cn('flex flex-col bg-black/20 rounded-sm', local.class)}
      {...others}
    >
      <For each={local.items}>
        {(item) => (
          <li>
            <local.item value={item} />
          </li>
        )}
      </For>
    </ul>
  );
};
