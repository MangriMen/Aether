import type { Component, ComponentProps } from 'solid-js';

import { createMemo, createSignal, For, Show, splitProps } from 'solid-js';

import type { Plugin, PluginMetadata } from '@/entities/plugins';

import { cn } from '@/shared/lib';
import { Separator } from '@/shared/ui';

import { PluginCard } from './PluginCard';
import { PluginInfoCard } from './PluginInfoCard';

export type PluginsListProps = {
  isLoading: boolean;
  plugins?: Plugin[];
} & ComponentProps<'div'>;

export const PluginsList: Component<PluginsListProps> = (props) => {
  const [local, others] = splitProps(props, ['plugins', 'class']);

  const [selectedPluginId, setSelectedPluginId] = createSignal<
    null | PluginMetadata['id']
  >(null);

  const selectedPlugin = createMemo(() =>
    local.plugins?.find(
      (plugin) => plugin.manifest.metadata.id === selectedPluginId(),
    ),
  );

  return (
    <div class={cn('flex size-full', local.class)} {...others}>
      <div
        class={cn('flex-1 basis-72 min-w-72', {
          'max-w-72': !!selectedPluginId(),
        })}
      >
        <For each={local.plugins}>
          {(plugin) => (
            <PluginCard
              isSelected={plugin.manifest.metadata.id === selectedPluginId()}
              onClick={() => setSelectedPluginId(plugin.manifest.metadata.id)}
              plugin={plugin}
              role='button'
            />
          )}
        </For>
      </div>

      <Show when={selectedPlugin()}>
        {(plugin) => (
          <>
            <Separator class='mx-2' orientation='vertical' />
            <PluginInfoCard class='my-2 flex-1 basis-full' plugin={plugin()} />
          </>
        )}
      </Show>
    </div>
  );
};
