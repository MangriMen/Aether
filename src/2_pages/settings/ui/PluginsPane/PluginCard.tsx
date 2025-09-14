import { type Component, type ComponentProps, splitProps } from 'solid-js';

import type { Plugin } from '@/entities/plugins';

import { cn } from '@/shared/lib';
import { Image } from '@/shared/ui';

export type PluginCardProps = {
  isSelected?: boolean;
  plugin: Plugin;
} & ComponentProps<'div'>;

export const PluginCard: Component<PluginCardProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'isSelected', 'class']);

  return (
    <div
      class={cn(
        'flex w-full items-center gap-2 text-sm px-2 py-1 rounded-md hover:bg-secondary/50 data-[active]:bg-secondary data-[active]:hover:bg-secondary data-[disabled]:text-muted-foreground',
        local.class,
      )}
      data-active={local.isSelected ? '' : undefined}
      data-disabled={!local.plugin.enabled ? '' : undefined}
      {...others}
    >
      <Image class='h-12 w-max' />
      <div class='flex flex-col truncate font-normal'>
        <span class='font-medium'>{local.plugin.manifest.metadata.name}</span>
        <span class='w-full truncate'>
          {local.plugin.manifest.metadata.description}
        </span>
        <span class='w-full truncate font-medium'>
          {local.plugin.manifest.metadata.authors?.join(', ')}
        </span>
      </div>
    </div>
  );
};
