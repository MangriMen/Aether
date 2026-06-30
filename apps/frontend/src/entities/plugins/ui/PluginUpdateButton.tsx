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
  useCheckPluginUpdates,
  useDisablePlugin,
  useEnablePlugin,
  useUpdatesData,
  useUpdatePlugin,
} from '../model/queries';

export type PluginUpdateButtonProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

/**
 * Update button shown in plugin details action bar.
 * Reads cached update data; user must click "Check for updates" to fetch.
 * After update succeeds, switches to a "Restart" button
 * that disables and re-enables the plugin to reload it.
 */
export const PluginUpdateButton: Component<PluginUpdateButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const pluginId = () => props.plugin.manifest.metadata.id;
  const updates = useUpdatesData(pluginId);
  const checkUpdates = useCheckPluginUpdates();
  const updatePlugin = useUpdatePlugin();
  const disablePlugin = useDisablePlugin();
  const enablePlugin = useEnablePlugin();

  const [needsRestart, setNeedsRestart] = createSignal(false);

  const hasChecked = createMemo(() => updates.data != null);
  const hasUpdate = createMemo(() => updates.data?.has_update ?? false);
  const showRestart = createMemo(() => needsRestart());

  const isBusy = createMemo(
    () =>
      checkUpdates.isPending ||
      updatePlugin.isPending ||
      disablePlugin.isPending ||
      enablePlugin.isPending,
  );

  const handleCheckUpdates = () => {
    checkUpdates.mutate(pluginId());
  };

  const handleUpdate = () => {
    const isCurrentlyLoaded = props.plugin.state === 'Loaded';
    updatePlugin.mutateAsync({ id: pluginId() }).then(() => {
      if (isCurrentlyLoaded) {
        setNeedsRestart(true);
      }
    });
  };

  const handleRestart = async () => {
    try {
      await disablePlugin.mutateAsync(pluginId());
      await enablePlugin.mutateAsync(pluginId());
      setNeedsRestart(false);
    } catch {
      // Error handled by mutations
    }

    try {
      await updates.refetch();
    } catch {
      /* empty */
    }
  };

  return (
    <>
      <Show when={!hasChecked() && !showRestart()}>
        <Button
          size='sm'
          onClick={handleCheckUpdates}
          disabled={isBusy()}
          loading={checkUpdates.isPending}
        >
          {t('plugins.checkUpdates')}
        </Button>
      </Show>

      <Show when={hasUpdate() || showRestart()}>
        <Button
          size='sm'
          onClick={showRestart() ? handleRestart : handleUpdate}
          disabled={isBusy()}
          loading={isBusy()}
        >
          {showRestart() ? t('plugins.restartPlugin') : t('plugins.update')}
        </Button>
      </Show>

      <Show when={hasChecked() && !hasUpdate() && !showRestart()}>
        <Button
          size='sm'
          variant='ghost'
          onClick={handleCheckUpdates}
          disabled={isBusy()}
          loading={checkUpdates.isPending}
        >
          {t('plugins.upToDate')}
        </Button>
      </Show>
    </>
  );
};
