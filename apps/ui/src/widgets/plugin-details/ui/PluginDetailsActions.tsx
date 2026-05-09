import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { Plugin } from '@/entities/plugins';

import { TogglePluginButton } from '@/features/toggle-plugin-button';

export type PluginDetailsActionsProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetailsActions: Component<PluginDetailsActionsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  return (
    <div class='flex' {...others}>
      <TogglePluginButton plugin={local.plugin} />
    </div>
  );
};
