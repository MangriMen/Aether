import type { Component } from 'solid-js';

import { For, splitProps } from 'solid-js';

import type { ToggleGroupRootProps } from '@/shared/ui-kit';
import type { Option } from '@/shared/utils';

import { useTranslation } from '@/shared/i18n';
import { ToggleGroup, ToggleGroupItem } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

export type LoaderVersionTypeChipsToggleGroupProps = Exclude<
  ToggleGroupRootProps,
  'onChange'
> & {
  loaderTypes: Option<string>[];
};

export const LoaderVersionTypeChipsToggleGroup: Component<
  LoaderVersionTypeChipsToggleGroupProps
> = (props) => {
  const [local, others] = splitProps(props, ['loaderTypes', 'class']);

  const [{ t }] = useTranslation();

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
