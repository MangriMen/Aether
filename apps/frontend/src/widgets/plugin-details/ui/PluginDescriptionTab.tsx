import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';

import type { PluginDetailsTabProps } from '../model';

export type PluginDescriptionTabProps = ComponentProps<'div'> &
  PluginDetailsTabProps;

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
