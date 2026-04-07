import { useQueryClient } from '@tanstack/solid-query';
import {
  createMemo,
  onMount,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { prefetchPluginSettings, type Plugin } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { Image, Separator } from '@/shared/ui';

import { PluginDetailsActions } from './PluginDetailsActions';
import { PluginDetailsBody } from './PluginDetailsBody';
import { PluginDetailsInfo } from './PluginDetailsInfo';

export type PluginDetailsProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetails: Component<PluginDetailsProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const queryClient = useQueryClient();

  const isSettingsDisabled = createMemo(() => {
    const state = local.plugin.state;
    return state === 'Loaded' || state === 'Loading' || state === 'Unloading';
  });

  const pluginId = () => local.plugin.manifest.metadata.id;

  onMount(() => prefetchPluginSettings(queryClient, pluginId));

  return (
    <div class={cn('flex flex-col gap-4', local.class)} {...others}>
      <div class='ml-4 flex items-center gap-4'>
        <Image class='h-[124px] w-max' />
        <div class='flex flex-col gap-2'>
          <PluginDetailsInfo plugin={local.plugin} />
          <PluginDetailsActions plugin={local.plugin} />
        </div>
      </div>

      <Separator />

      <PluginDetailsBody
        class='ml-4 grow'
        plugin={local.plugin}
        isSettingsDisabled={isSettingsDisabled()}
      />
    </div>
  );
};
