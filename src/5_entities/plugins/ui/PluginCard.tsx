import type { Component } from 'solid-js';

import { createMemo, splitProps, type ComponentProps } from 'solid-js';

import type { Plugin } from '@/entities/plugins';

import { cn } from '@/shared/lib';
import { Image } from '@/shared/ui';

export type PluginCardProps = ComponentProps<'div'> & {
  plugin: Plugin;
  isSelected?: boolean;
};

export const PluginCard: Component<PluginCardProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'isSelected', 'class']);

  const metadata = createMemo(() => local.plugin.manifest.metadata);
  const authors = createMemo(() => metadata().authors?.join(', '));

  const isDisabled = createMemo(() => {
    const state = local.plugin.state;

    return state === 'Failed' || state === 'NotLoaded';
  });

  return (
    <div
      class={cn(
        'flex w-full items-center gap-2 text-sm px-2 py-1 rounded-md hover:bg-secondary/50 data-[active]:bg-secondary data-[active]:hover:bg-secondary data-[disabled]:text-muted-foreground',
        local.class,
      )}
      data-disabled={isDisabled() ? '' : undefined}
      data-active={local.isSelected ? '' : undefined}
      {...others}
    >
      <Image class='h-12 w-max' />
      <div class='flex flex-col truncate font-normal'>
        <span class='font-medium'>{metadata().name}</span>
        <span class='w-full truncate'>{metadata().description}</span>
        <span class='w-full truncate font-medium'>{authors()}</span>
      </div>
    </div>
  );
};
