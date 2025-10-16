import type { Component, ComponentProps } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { PluginsList, type Plugin } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { Separator } from '@/shared/ui';

import { usePluginSelection } from '../lib';
import { PluginDetails } from './PluginDetails';

export type PluginsViewProps = ComponentProps<'div'> & {
  plugins?: Plugin[];
  isLoading?: boolean;
};

export const PluginsView: Component<PluginsViewProps> = (props) => {
  const [local, others] = splitProps(props, ['plugins', 'isLoading', 'class']);

  const { selectedPluginId, selectedPlugin, selectPlugin } = usePluginSelection(
    () => local.plugins,
  );

  return (
    <div class={cn('flex size-full', local.class)} {...others}>
      <PluginsList
        class={cn('flex-1 basis-72 min-w-72', {
          'max-w-72': !!selectedPluginId(),
        })}
        plugins={local.plugins}
        isLoading={local.isLoading}
        selectedPluginId={selectedPluginId()}
        onPluginSelect={selectPlugin}
      />

      <Show when={selectedPlugin()}>
        {(plugin) => (
          <>
            <Separator class='mx-2' orientation='vertical' />
            <PluginDetails class='flex-1 basis-full' plugin={plugin()} />
          </>
        )}
      </Show>
    </div>
  );
};
