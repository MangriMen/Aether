import type { Component, ComponentProps } from 'solid-js';

import { relaunch } from '@tauri-apps/plugin-process';
import { createMemo, Show } from 'solid-js';

import { useCheckUpdate, useInstallUpdate } from '@/entities/updates';
import { checkIsUpdateAvailable } from '@/entities/updates/model';
import { dayjs, formatTime } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  MarkdownRenderer,
  SettingsEntry,
  showToast,
} from '@/shared/ui';

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

  const { isUpdating, updateAndRestart: installUpdate } = useInstallUpdate();

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

  const date = createMemo(() =>
    update.data?.date ? formatTime(dayjs(update.data.date)) : '',
  );

  const markdownText =
    '### Bug fixes\n* Fix shaders/resource packs in modded instances\n\nSee the assets to download this version and install.';

  return (
    <div>
      <SettingsEntry
        class='items-start'
        title={t('settings.checkForUpdates')}
        description={
          <Show
            when={isUpdateAvailable()}
            fallback={t('settings.checkForUpdatesDescriptionNoUpdates')}
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
                  {t('settings.releaseDate')}: {date()}
                </span>
              </Show>
            </div>
          </Show>
        }
        {...props}
      >
        <div class='mt-6 flex h-full min-w-max flex-col justify-start gap-2'>
          <Show
            when={isUpdateAvailable()}
            fallback={
              <Button loading={update.isFetching} onClick={checkUpdates}>
                {t('settings.checkForUpdates')}
              </Button>
            }
          >
            <Button loading={isUpdating()} onClick={handleInstallUpdate}>
              {t('settings.installAndRestart')}
            </Button>
          </Show>
        </div>
      </SettingsEntry>
      <Show when={update.data?.body}>
        {(_body) => (
          <div class='mt-2 flex flex-col gap-2'>
            <span class='text-xl font-medium'>{t('settings.whatsNew')}</span>
            <MarkdownRenderer
              class='min-w-full rounded-md border-2 border-secondary-dark p-4 dark:border-secondary'
              children={markdownText}
            />
          </div>
        )}
      </Show>
    </div>
  );
};
