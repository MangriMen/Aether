import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { PluginApiCompatibilityBadge, type Plugin } from '@/entities/plugins';
import { cn } from '@/shared/lib';

export type PluginDetailsInfoProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetailsInfo: Component<PluginDetailsInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const metadata = createMemo(() => local.plugin.manifest.metadata);
  const authorsStr = createMemo(() => metadata().authors?.join(', '));

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <div class='flex items-end gap-2'>
        <h2 class='text-xl font-bold'>{metadata().name}</h2>
        <div class='flex items-end gap-1'>
          <span class='text-muted-foreground'>{metadata().version}</span>
          <PluginApiCompatibilityBadge
            class='mb-[3px]'
            apiVersion={local.plugin.manifest.api.version}
          />
        </div>
      </div>
      <div class='flex gap-4'>
        <span>{authorsStr()}</span>
      </div>
      <span class='my-1'>{metadata().description}</span>
    </div>
  );
};
