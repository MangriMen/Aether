import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { usePluginReadme } from '@/features/plugins';
import { MarkdownRenderer } from '@/shared/markdown';
import { cn } from '@/shared/utils';

import type { PluginDetailsTabProps } from '../model/pluginDetailsTabs';

export type PluginDescriptionTabProps = ComponentProps<'div'> &
  PluginDetailsTabProps;

/** The plugin's README. The text is the plugin author's: it is sanitized like any other. */
export const PluginDescriptionTab: Component<PluginDescriptionTabProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const readme = usePluginReadme(
    () => local.plugin.manifest.metadata.id,
    () => local.plugin.manifest.metadata.version,
  );

  return (
    <div class={cn('overflow-y-auto', local.class)} {...others}>
      <Show when={readme.data}>
        {(text) => (
          <div class='rounded-md bg-card/card max-w-full overflow-auto border'>
            <MarkdownRenderer class='p-4' children={text()} />
          </div>
        )}
      </Show>
    </div>
  );
};
