import type { Component, ComponentProps } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import type { PluginDetails } from '@/widgets/plugin-details';

import { PluginsList, type Plugin } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useThemeContext } from '@/shared/model';
import { Separator } from '@/shared/ui';

import { usePluginSelection } from '../lib';

export type PluginsViewProps = ComponentProps<'div'> & {
  pluginDetails: Component<ComponentProps<typeof PluginDetails>>;
  plugins?: Plugin[];
  isLoading?: boolean;
};

export const PluginsView: Component<PluginsViewProps> = (props) => {
  const [local, others] = splitProps(props, [
    'plugins',
    'isLoading',
    'pluginDetails',
    'class',
  ]);

  const [themeContext] = useThemeContext();

  const {
    selectedPluginId,
    selectedPlugin,
    hasSelectedPlugin,
    selectPluginAnimated,
    unselectPluginAnimated,
  } = usePluginSelection(
    () => local.plugins,
    () => !themeContext.disableAnimations,
  );

  return (
    <div
      class={cn('flex size-full relative overflow-hidden', local.class)}
      style={{ '--min-list-width': '18rem' }}
      {...others}
    >
      <PluginsList
        class={cn(
          'shrink-0 grow min-w-[var(--min-list-width)] transition-[max-width] duration-200 max-w-full',
          {
            'max-w-[var(--min-list-width)]': hasSelectedPlugin(),
          },
        )}
        plugins={local.plugins}
        isLoading={local.isLoading}
        selectedPluginId={selectedPluginId()}
        onPluginSelect={selectPluginAnimated}
      />

      <div
        class={cn(
          'flex right-0 inset-y-0 absolute left-[var(--min-list-width)] transition-[transform,opacity] duration-200 translate-x-full opacity-0',
          {
            'translate-x-0 opacity-100': hasSelectedPlugin(),
          },
        )}
        onTransitionEnd={unselectPluginAnimated}
      >
        <Separator class='mx-2 shrink-0' orientation='vertical' />
        <Show when={selectedPlugin()}>
          {(plugin) => <local.pluginDetails class='flex-1' plugin={plugin()} />}
        </Show>
      </div>
    </div>
  );
};
