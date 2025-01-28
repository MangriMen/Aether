import type { Component } from 'solid-js';
import { For, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { ToggleGroup, ToggleGroupItem } from '@/shared/ui';

// eslint-disable-next-line boundaries/element-types
import { useTranslate } from '@/app/model';

import type { LoaderVersionTypeChipsToggleGroupProps } from '.';

export const LoaderVersionTypeChipsToggleGroup: Component<
  LoaderVersionTypeChipsToggleGroupProps
> = (props) => {
  const [local, others] = splitProps(props, ['loaderTypes', 'class']);

  const [{ t }] = useTranslate();

  return (
    <ToggleGroup class={cn('justify-start', local.class)} {...others}>
      <For each={local.loaderTypes}>
        {(versionType) => (
          <ToggleGroupItem value={versionType.value}>
            {t(
              `createInstance.loaderVersion${versionType.name as 'Stable' | 'Latest' | 'Other'}`,
            )}
          </ToggleGroupItem>
        )}
      </For>
    </ToggleGroup>
  );
};
