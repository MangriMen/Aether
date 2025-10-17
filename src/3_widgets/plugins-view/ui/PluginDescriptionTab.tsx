import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { Plugin } from '@/entities/plugins';

import { cn } from '@/shared/lib';

export type PluginDescriptionTabProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDescriptionTab: Component<PluginDescriptionTabProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const description = createMemo(
    () => local.plugin.manifest.metadata.description,
  );

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <p>{description()}</p>
    </div>
  );
};
