import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { type Plugin } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { Separator } from '@/shared/ui';

import { PluginDetailsActions } from './PluginDetailsActions';
import { PluginDetailsInfo } from './PluginDetailsInfo';
import { PluginSettings } from './PluginSettings';

export type PluginDetailsProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetails: Component<PluginDetailsProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const isSettingsDisabled = createMemo(() => {
    const state = local.plugin.state;
    return state === 'Loaded' || state === 'Loading' || state === 'Unloading';
  });

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <PluginDetailsInfo plugin={local.plugin} />
      <PluginDetailsActions plugin={local.plugin} />

      <Separator />

      <p class='pb-1'>{local.plugin.manifest.metadata.description}</p>

      <PluginSettings plugin={local.plugin} disabled={isSettingsDisabled()} />
    </div>
  );
};
