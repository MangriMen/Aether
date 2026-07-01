import type { Component } from 'solid-js';

import { createMemo, splitProps, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Image } from '@/shared/ui';

import type { Plugin } from '..';

import { PluginApiCompatibilityBadge } from './PluginApiCompatibilityBadge';
import { PluginUpdateBadge } from './PluginUpdateBadge';

export type PluginCardBaseProps = {
  plugin: Plugin;
  isSelected?: boolean;
};

export type PluginCardProps = ComponentProps<'div'> & PluginCardBaseProps;

export const PluginCard: Component<PluginCardProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'isSelected', 'class']);

  const metadata = createMemo(() => local.plugin.manifest.metadata);
  const authors = createMemo(() => metadata().authors?.join(', '));

  const isDisabled = createMemo(() => {
    const state = local.plugin.state;

    return (
      state === 'Failed' || state === 'NotLoaded' || state === 'Incompatible'
    );
  });

  return (
    <div
      class={cn(
        'flex w-full items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-card/hover data-[active]:bg-secondary/secondary data-[active]:hover:bg-secondary/hover data-[disabled]:text-muted-foreground',
        local.class,
      )}
      data-disabled={isDisabled() ? '' : undefined}
      data-active={local.isSelected ? '' : undefined}
      {...others}
    >
      <Image class='h-14 min-h-14 w-max min-w-max' />
      <div class='flex flex-col truncate font-normal'>
        <span class='flex items-center gap-1 text-base font-medium'>
          {metadata().name}
          <PluginApiCompatibilityBadge
            class='text-sm'
            apiVersion={local.plugin.manifest.api.version}
          />
          <PluginUpdateBadge plugin={local.plugin} />
        </span>
        <span class='w-full truncate'>{metadata().description}</span>
        <span class='w-full truncate font-medium'>{authors()}</span>
      </div>
    </div>
  );
};
