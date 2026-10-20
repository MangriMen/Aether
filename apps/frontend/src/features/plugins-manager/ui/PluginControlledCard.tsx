import { splitProps, type Component } from 'solid-js';

import type { PluginCardBaseProps } from '@/features/plugins';

import {
  PluginCard,
  PluginContextMenu,
  useDisablePlugin,
  useForceEnablePluginWithDialog,
  usePluginStates,
  useRemovePlugin,
} from '@/features/plugins';
import { logError } from '@/shared/log';
import { ContextMenuTrigger } from '@/shared/ui-kit';

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
    try {
      await removePlugin.mutateAsync(local.plugin.manifest.metadata.id);
    } catch (error) {
      logError('Failed to remove the plugin:', error);
    }
  };

  return (
    <PluginContextMenu
      onToggleEnabled={handleToggleEnabled}
      onRemove={handleRemove}
      isDisabled={isDisabled()}
      isLoading={isLoading()}
      isEnabled={isEnabled()}
    >
      <ContextMenuTrigger
        as={PluginCard}
        data-context-menu-safe='true'
        plugin={local.plugin}
        {...others}
      />
    </PluginContextMenu>
  );
};
