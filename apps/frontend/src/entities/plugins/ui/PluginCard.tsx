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
        `
          gap-2 rounded-md px-2 py-1 text-sm
          hover:bg-card/hover
          data-active:bg-secondary/secondary
          data-active:hover:bg-secondary/hover
          data-disabled:text-muted-foreground
          flex w-full items-center
        `,
        local.class,
      )}
      data-disabled={isDisabled() ? '' : undefined}
      data-active={local.isSelected ? '' : undefined}
      {...others}
    >
      <Image class='h-14 min-h-14 w-max min-w-max' />
      <div class='font-normal flex flex-col truncate'>
        <span class='gap-1 text-base font-medium flex items-center'>
          {metadata().name}
          <PluginApiCompatibilityBadge
            class='text-sm'
            apiVersion={local.plugin.manifest.api.version}
          />
          <PluginUpdateBadge plugin={local.plugin} />
        </span>
        <span class='w-full truncate'>{metadata().description}</span>
        <span class='font-medium w-full truncate'>{authors()}</span>
      </div>
    </div>
  );
};
