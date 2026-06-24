import { splitProps, type Component } from 'solid-js';

import type { PluginCardBaseProps } from '@/entities/plugins';

import {
  PluginCard,
  PluginContextMenu,
  useDisablePlugin,
  usePluginStates,
  useRemovePlugin,
} from '@/entities/plugins';
import { useForceEnablePluginWithDialog } from '@/features/toggle-plugin-button';
import { ContextMenuTrigger } from '@/shared/ui';

export type PluginControlledCardProps = {
  class?: string;
} & PluginCardBaseProps;

export const PluginControlledCard: Component<PluginControlledCardProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin']);

  const { isDisabled, isLoading, isEnabled } = usePluginStates(
    () => local.plugin.state,
  );

  const { tryEnable } = useForceEnablePluginWithDialog();
  const disablePlugin = useDisablePlugin();

  const handleToggleEnabled = async () => {
    if (isLoading()) {
      return;
    }

    try {
      const id = local.plugin.manifest.metadata.id;
      if (isEnabled()) {
        await disablePlugin.mutateAsync(id);
      } else {
        await tryEnable(local.plugin);
      }
    } catch {
      /* empty */
    }
  };

  const removePlugin = useRemovePlugin();

  const handleRemove = async () => {
    await removePlugin.mutateAsync(local.plugin.manifest.metadata.id);
  };

  return (
    <PluginContextMenu
      onToggleEnabled={handleToggleEnabled}
      onRemove={handleRemove}
      isDisabled={isDisabled()}
      isLoading={isLoading()}
      isEnabled={isEnabled()}
    >
      <ContextMenuTrigger as={PluginCard} plugin={local.plugin} {...others} />
    </PluginContextMenu>
  );
};
