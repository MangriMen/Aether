import { splitProps, type Component, type ComponentProps, For } from 'solid-js';

import { cn } from '@/shared/lib';
import { DelayedShow, SkeletonList } from '@/shared/ui';

import type { Plugin, PluginMetadata } from '../model';

import { PluginCard } from './PluginCard';

export type PluginsListProps = ComponentProps<'div'> & {
  plugins?: Plugin[];
  isLoading?: boolean;
  selectedPluginId?: string;
  onPluginSelect?: (pluginId: PluginMetadata['id']) => void;
};

export const PluginsList: Component<PluginsListProps> = (props) => {
  const [local, others] = splitProps(props, [
    'plugins',
    'isLoading',
    'selectedPluginId',
    'onPluginSelect',
    'class',
  ]);

  return (
    <div class={cn('flex flex-col gap-1', local.class)} {...others}>
      <DelayedShow
        when={!local.isLoading}
        fallback={<SkeletonList itemsCount={3} height={60} radius={6} />}
      >
        <For each={local.plugins}>
          {(plugin) => (
            <button
              class='w-full text-left'
              onClick={() =>
                local.onPluginSelect?.(plugin.manifest.metadata.id)
              }
            >
              <PluginCard
                plugin={plugin}
                isSelected={
                  local.selectedPluginId === plugin.manifest.metadata.id
                }
              />
            </button>
          )}
        </For>
      </DelayedShow>
    </div>
  );
};
