import { Component, For, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { ToggleGroup, ToggleGroupItem } from '@/shared/ui';

import { SelectLoaderChipsProps } from '.';

export const SelectLoaderChips: Component<SelectLoaderChipsProps> = (props) => {
  const [local, others] = splitProps(props, ['loaders', 'class']);

  return (
    <ToggleGroup class={cn('justify-start', local.class)} {...others}>
      <For each={local.loaders}>
        {(modLoader) => (
          <ToggleGroupItem value={modLoader.value}>
            {modLoader.name}
          </ToggleGroupItem>
        )}
      </For>
    </ToggleGroup>
  );
};
