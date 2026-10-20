import {
  createMemo,
  Show,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Button } from '@/shared/ui-kit';

import type { Plugin } from '../model/plugin';

import {
  useCheckPluginUpdates,
  useUpdatesData,
  useUpdatePlugin,
} from '../api/pluginQueries';

export type PluginUpdateButtonProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

/**
 * Update button shown in plugin details action bar.
 * Reads cached update data; user must click "Check for updates" to fetch.
 * The core reloads an enabled plugin itself once it is updated.
 */
export const PluginUpdateButton: Component<PluginUpdateButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const pluginId = () => props.plugin.manifest.metadata.id;
  const updates = useUpdatesData(pluginId);
  const checkUpdates = useCheckPluginUpdates();
  const updatePlugin = useUpdatePlugin();

  const hasChecked = createMemo(() => updates.data != null);
  const hasUpdate = createMemo(() => updates.data?.has_update ?? false);

  const isBusy = createMemo(
    () => checkUpdates.isPending || updatePlugin.isPending,
  );

  const handleCheckUpdates = () => {
    checkUpdates.mutate(pluginId());
  };

  // Errors are shown by the mutation itself.
  const handleUpdate = () => {
    updatePlugin.mutate({ id: pluginId() });
  };

  return (
    <>
      <Show when={!hasChecked()}>
        <Button
          size='sm'
          onClick={handleCheckUpdates}
          disabled={isBusy()}
          loading={checkUpdates.isPending}
        >
          {t('plugins.checkUpdates')}
        </Button>
      </Show>

      <Show when={hasUpdate()}>
        <Button
          size='sm'
          onClick={handleUpdate}
          disabled={isBusy()}
          loading={isBusy()}
        >
          {t('plugins.update')}
        </Button>
      </Show>

      <Show when={hasChecked() && !hasUpdate()}>
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
