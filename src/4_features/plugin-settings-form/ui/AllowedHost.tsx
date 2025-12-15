import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';

import type { AllowedItemProps } from '../model/allowedItem';

export type AllowedHostProps = ComponentProps<'div'> &
  AllowedItemProps<string, string>;

export const AllowedHost: Component<AllowedHostProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'error',
    'leadingItems',
    'class',
  ]);

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <div class='group flex select-none items-center justify-between gap-2 rounded-sm px-2 py-1 hover:bg-secondary/50'>
        <span>{local.value}</span>
        <Show when={local.leadingItems}>
          {(leadingItems) => (
            <div class='flex items-center opacity-0 transition-opacity group-hover:opacity-100'>
              {leadingItems()}
            </div>
          )}
        </Show>
      </div>
      <span class='text-destructive'>{local.error}</span>
    </div>
  );
};
