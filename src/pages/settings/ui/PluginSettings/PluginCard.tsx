import { InstanceImage } from '@/entities/instances';
import type { PluginMetadata } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

export type PluginCardProps = ComponentProps<'div'> & {
  plugin: PluginMetadata;
  isActive?: boolean;
};

export const PluginCard: Component<PluginCardProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'isActive', 'class']);

  return (
    <div
      class={cn(
        'flex w-full items-center gap-2 text-sm px-2 py-1 rounded-md hover:bg-secondary/50 data-[active]:bg-secondary data-[active]:hover:bg-secondary',
        local.class,
      )}
      data-active={local.isActive ? '' : undefined}
      {...others}
    >
      <InstanceImage class='h-12 w-max' />
      <div class='flex flex-col truncate font-normal'>
        <span class='font-medium'>{local.plugin.plugin.name}</span>
        <span class='w-full truncate'>{local.plugin.plugin.description}</span>
        <span class='w-full truncate font-medium'>
          {local.plugin.plugin.authors?.join(', ')}
        </span>
      </div>
    </div>
  );
};
