import { useQueryClient } from '@tanstack/solid-query';
import {
  onMount,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { pluginsCache, type Plugin } from '@/features/plugins';
import { Image, Separator } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { PluginDetailsActions } from './PluginDetailsActions';
import { PluginDetailsBody } from './PluginDetailsBody';
import { PluginDetailsInfo } from './PluginDetailsInfo';

export type PluginDetailsProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetails: Component<PluginDetailsProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const queryClient = useQueryClient();

  const pluginId = () => local.plugin.manifest.metadata.id;

  onMount(() => pluginsCache.prefetch.settings(queryClient, pluginId()));

  return (
    <div class={cn('gap-4 flex flex-col', local.class)} {...others}>
      <div class='ml-4 gap-4 flex items-center'>
        <Image class='h-31 w-max' />
        <div class='gap-2 flex flex-col'>
          <PluginDetailsInfo plugin={local.plugin} />
          <PluginDetailsActions plugin={local.plugin} />
        </div>
      </div>

      <Separator />

      <PluginDetailsBody class='ml-4' plugin={local.plugin} />
    </div>
  );
};
