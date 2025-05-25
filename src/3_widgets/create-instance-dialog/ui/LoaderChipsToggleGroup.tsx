import type { Component } from 'solid-js';
import { For, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import type { ToggleGroupRootProps } from '@/shared/ui';
import { ToggleGroup, ToggleGroupItem } from '@/shared/ui';

import type { ModLoader } from '@/5_entities/minecraft';
import type { Option } from '@/shared/model';

export type LoaderChipsToggleGroupProps = ToggleGroupRootProps & {
  loaders: Option<ModLoader>[];
  onChange: (value: ModLoader) => void;
};

export const LoaderChipsToggleGroup: Component<LoaderChipsToggleGroupProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['loaders', 'onChange', 'class']);

  const handleChange = (value: string | string[] | null) => {
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
