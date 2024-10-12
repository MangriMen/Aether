import { Component, For, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { ToggleGroup, ToggleGroupItem } from '@/shared/ui';

import { SelectLoaderTypeChipsProps } from '.';

export const SelectLoaderTypeChips: Component<SelectLoaderTypeChipsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['loaderTypes', 'class']);

  return (
    <ToggleGroup class={cn('justify-start', local.class)} {...others}>
      <For each={local.loaderTypes}>
        {(versionType) => (
          <ToggleGroupItem value={versionType.value}>
            {versionType.name}
          </ToggleGroupItem>
        )}
      </For>
    </ToggleGroup>
  );
};
