import type { Component } from 'solid-js';

import { For, splitProps } from 'solid-js';

import type { ModLoader } from '@/entities/minecraft';
import type { Option } from '@/shared/model';
import type { ToggleGroupRootProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { ToggleGroup, ToggleGroupItem } from '@/shared/ui';

export type LoaderChipsToggleGroupProps = {
  loaders: Option<ModLoader>[];
  onChange: (value: ModLoader) => void;
} & ToggleGroupRootProps;

export const LoaderChipsToggleGroup: Component<LoaderChipsToggleGroupProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['loaders', 'onChange', 'class']);

  const handleChange = (value: null | string | string[]) => {
    if (!value || typeof value === 'object') {
      return;
    }

    local.onChange?.(value as ModLoader);
  };

  return (
    <ToggleGroup
      class={cn('justify-start', local.class)}
      onChange={handleChange}
      {...others}
    >
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
