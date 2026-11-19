import {
  createMemo,
  createSignal,
  Show,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Button } from '@/shared/ui-kit';

import type { Plugin } from '../model/plugin';

import {
  useCheckPluginUpdates,
  useIsUpdatingPlugin,
  useUpdatesData,
} from '../api/pluginQueries';
import { PluginUpdateDialog } from './PluginUpdateDialog';

export type PluginUpdateButtonProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

/**
 * Update button shown in plugin details action bar.
 * Reads cached update data; user must click "Check for updates" to fetch.
 * The update goes through a dialog that shows what the latest release asks for; the core reloads
 * an enabled plugin itself once it is updated.
 */
export const PluginUpdateButton: Component<PluginUpdateButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const pluginId = () => props.plugin.manifest.metadata.id;
  const updates = useUpdatesData(pluginId);
  const checkUpdates = useCheckPluginUpdates();
  const isUpdating = useIsUpdatingPlugin(pluginId);

  // The release the dialog shows; null while it is closed.
  const [updateTag, setUpdateTag] = createSignal<string | null>(null);

  const hasChecked = createMemo(() => updates.data != null);
  const latestTag = createMemo(() =>
    updates.data?.hasUpdate ? (updates.data.latestTag ?? null) : null,
  );

  const handleCheckUpdates = () => {
    checkUpdates.mutate(pluginId());
  };

  const handleUpdate = () => {
    setUpdateTag(latestTag());
  };

  return (
    <>
      <Show when={!hasChecked()}>
        <Button
          size='sm'
          onClick={handleCheckUpdates}
          disabled={checkUpdates.isPending}
          loading={checkUpdates.isPending}
        >
          {t('plugins.checkUpdates')}
        </Button>
      </Show>

      <Show when={latestTag()}>
        <Button
          size='sm'
          onClick={handleUpdate}
          disabled={checkUpdates.isPending || isUpdating()}
          loading={isUpdating()}
        >
          {t('plugins.update')}
        </Button>
      </Show>

      <Show when={hasChecked() && !latestTag()}>
        <Button
          size='sm'
          variant='ghost'
          onClick={handleCheckUpdates}
          disabled={checkUpdates.isPending}
          loading={checkUpdates.isPending}
        >
          {t('plugins.upToDate')}
        </Button>
      </Show>

      <PluginUpdateDialog
        plugin={props.plugin}
        tag={updateTag()}
        onClose={() => setUpdateTag(null)}
      />
    </>
  );
};
