import type { Component, ComponentProps } from 'solid-js';

import { relaunch } from '@tauri-apps/plugin-process';
import { createMemo, Show } from 'solid-js';

import { useCheckUpdate, useInstallUpdate } from '@/entities/updates';
import { checkIsUpdateAvailable } from '@/entities/updates/model';
import { useTranslation } from '@/shared/model';
import { Button, SettingsEntry, showToast } from '@/shared/ui';

export type UpdateAppEntryProps = ComponentProps<'div'>;

export const UpdateAppEntry: Component<UpdateAppEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  const checkUpdates = () => {
    update.refetch();
  };

  const { installUpdate, isUpdating } = useInstallUpdate();

  const handleInstallUpdate = async () => {
    if (!update.data) {
      return;
    }

    try {
      await installUpdate(update.data);

      await relaunch();
    } catch {
      showToast({
        title: t('update.updateError'),
        variant: 'destructive',
      });
    }
  };

  return (
    <SettingsEntry
      class='items-start'
      description={
        <Show
          fallback={t('settings.checkForUpdatesDescriptionNoUpdates')}
          when={isUpdateAvailable()}
        >
          <div class='flex flex-col'>
            {t('settings.checkForUpdatesDescription')}
            <Show when={update.data?.version}>
              <span>
                {t('common.version')}: {update.data?.version}
              </span>
            </Show>
            <Show when={update.data?.date}>
              <span>
                {t('settings.releaseDate')}: {update.data?.date}
              </span>
            </Show>
          </div>
        </Show>
      }
      title={t('settings.checkForUpdates')}
      {...props}
    >
      <div class='flex h-full flex-col justify-center gap-2'>
        <Show
          fallback={
            <Button
              class='h-auto max-h-none'
              loading={update.isFetching}
              onClick={checkUpdates}
            >
              {t('settings.checkForUpdates')}
            </Button>
          }
          when={isUpdateAvailable()}
        >
          <Button
            class='h-auto max-h-none'
            loading={isUpdating()}
            onClick={handleInstallUpdate}
          >
            {t('settings.installAndRestartApp')}
          </Button>
        </Show>
      </div>
    </SettingsEntry>
  );
};
