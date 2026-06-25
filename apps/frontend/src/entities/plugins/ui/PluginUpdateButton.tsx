import {
  createMemo,
  createSignal,
  Show,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

import type { Plugin } from '../model/plugin';

import {
  useCheckForUpdates,
  useDisablePlugin,
  useEnablePlugin,
  useUpdatePlugin,
} from '../model/queries';

export type PluginUpdateButtonProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

/**
 * Update button shown in plugin details action bar.
 * After update succeeds, switches to a "Restart" button
 * that disables and re-enables the plugin to reload it.
 */
export const PluginUpdateButton: Component<PluginUpdateButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const pluginId = () => props.plugin.manifest.metadata.id;
  const updates = useCheckForUpdates(pluginId);
  const updatePlugin = useUpdatePlugin();
  const disablePlugin = useDisablePlugin();
  const enablePlugin = useEnablePlugin();

  const [needsRestart, setNeedsRestart] = createSignal(false);

  const hasUpdate = createMemo(() => updates.data?.has_update ?? false);
  const showRestart = createMemo(
    () => needsRestart() || updatePlugin.isSuccess,
  );

  const handleUpdate = () => {
    updatePlugin.mutate(
      { id: pluginId() },
      {
        onSuccess: () => setNeedsRestart(true),
      },
    );
  };

  const handleRestart = async () => {
    try {
      await disablePlugin.mutateAsync(pluginId());
      await enablePlugin.mutateAsync(pluginId());
      setNeedsRestart(false);
    } catch {
      // Error handled by mutations
    }
  };

  return (
    <Show when={hasUpdate() || showRestart()}>
      <Button
        size='sm'
        onClick={showRestart() ? handleRestart : handleUpdate}
        disabled={updatePlugin.isPending}
        loading={disablePlugin.isPending || enablePlugin.isPending}
      >
        {showRestart()
          ? t('plugins.restartPlugin')
          : t('plugins.updateToLatest')}
      </Button>
    </Show>
  );
};
