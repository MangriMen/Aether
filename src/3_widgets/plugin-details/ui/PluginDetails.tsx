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
import { PluginDetailsBody } from './PluginDetailsBody';
import { PluginDetailsInfo } from './PluginDetailsInfo';

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

      <PluginDetailsBody
        class='grow'
        plugin={local.plugin}
        isSettingsDisabled={isSettingsDisabled()}
      />
    </div>
  );
};
