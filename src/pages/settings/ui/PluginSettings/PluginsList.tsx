import type { Plugin, PluginInfo } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import type { ReactiveMap } from '@solid-primitives/map';
import { createMemo, createSignal, For, Show, splitProps } from 'solid-js';
import type { Component, ComponentProps } from 'solid-js';
import { PluginCard } from './PluginCard';
import { Separator } from '@/shared/ui';
import { PluginInfoCard } from './PluginInfoCard';

export type PluginsListProps = ComponentProps<'div'> & {
  plugins: ReactiveMap<string, Plugin>;
};

export const PluginsList: Component<PluginsListProps> = (props) => {
  const [local, others] = splitProps(props, ['plugins', 'class']);

  const pluginsValues = createMemo(() => Array.from(local.plugins.values()));

  const [selectedPluginId, setSelectedPluginId] = createSignal<
    PluginInfo['id'] | null
  >(null);

  const selectedPlugin = () => {
    const id = selectedPluginId();
    return id ? local.plugins.get(id) : null;
  };

  return (
    <div class={cn('flex size-full', local.class)} {...others}>
      <div
        class={cn('flex-1 basis-72 min-w-72', {
          'max-w-72': !!selectedPluginId(),
        })}
      >
        <For each={pluginsValues()}>
          {(plugin) => (
            <PluginCard
              role='button'
              isSelected={plugin.metadata.plugin.id === selectedPluginId()}
              plugin={plugin}
              onClick={() => setSelectedPluginId(plugin.metadata.plugin.id)}
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
