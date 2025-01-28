import { emit } from '@tauri-apps/api/event';
import { relaunch } from '@tauri-apps/plugin-process';
import type { DownloadEvent } from '@tauri-apps/plugin-updater';
import type { Component, ComponentProps } from 'solid-js';
import { createSignal, Show } from 'solid-js';

import { Button, showToast } from '@/shared/ui';

import type { LoadingPayload } from '@/entities/minecraft';
import { LoadingBarTypeEnum } from '@/entities/minecraft';
import { updateResource } from '@/entities/update';


import { useTranslate } from '@/app/model';
import { getVersion } from '@tauri-apps/api/app';

import { SettingsEntry } from '../SettingsEntry';

export type UpdateAppEntryProps = ComponentProps<'div'>;

const UpdateAppEntry: Component<UpdateAppEntryProps> = (props) => {
  const [update, { refetch }] = updateResource;
  const [{ t }] = useTranslate();

  const checkUpdates = () => {
    refetch();
  };

  const [isUpdating, setIsUpdating] = createSignal(false);

  let downloaded: number = 0;
  let contentLength: number | undefined = 0;

  const handleUpdatingEvent = async (event: DownloadEvent) => {
    switch (event.event) {
      case 'Started': {
        contentLength = event.data.contentLength;

        const payload: LoadingPayload = {
          event: {
            type: LoadingBarTypeEnum.LauncherUpdate,
            version: update()?.version ?? '',
            current_version: await getVersion(),
          },
          loaderUuid: '',
          fraction: 0,
          message: 'Start updating launcher',
        };

        emit('loading', payload);
        break;
      }
      case 'Progress': {
        downloaded += event.data.chunkLength;
        if (contentLength) {
          const payload: LoadingPayload = {
            event: {
              type: LoadingBarTypeEnum.LauncherUpdate,
              version: update()?.version ?? '',
              current_version: await getVersion(),
            },
            loaderUuid: '',
            fraction: downloaded / contentLength,
            message: 'Updating launcher',
          };

          emit('loading', payload);
        }
        break;
      }
      case 'Finished': {
        const payload: LoadingPayload = {
          event: {
            type: LoadingBarTypeEnum.LauncherUpdate,
            version: update()?.version ?? '',
            current_version: await getVersion(),
          },
          loaderUuid: '',
          fraction: null,
          message: 'Finished updating',
        };

        emit('loading', payload);
        break;
      }
    }
  };

  const downloadAndInstallUpdate = async () => {
    if (!update()?.available) {
      return;
    }

    downloaded = 0;
    contentLength = 0;

    try {
      setIsUpdating(true);
      await update()?.downloadAndInstall(handleUpdatingEvent);

      await relaunch();
    } catch {
      setIsUpdating(false);
      showToast({
        title: 'Error updating launcher',
        variant: 'destructive',
      });
    }
  };

  return (
    <SettingsEntry
      class='items-start'
      title={t('settings.checkForUpdates')}
      description={
        <Show
          when={update()?.available}
          fallback={t('settings.checkForUpdatesDescriptionNoUpdates')}
        >
          <div class='flex flex-col'>
            {t('settings.checkForUpdatesDescription')}
            <span>
              {t('common.version')}: {update()?.version}
            </span>
            <Show when={update()?.date}>
              <span>
                {t('settings.releaseDate')}: {update()?.date}
              </span>
            </Show>
          </div>
        </Show>
      }
      {...props}
    >
      <div class='flex h-full flex-col justify-center gap-2'>
        <Show
          when={update()?.available}
          fallback={
            <Button
              loading={update.loading}
              disabled={isUpdating()}
              onClick={checkUpdates}
            >
              {t('settings.checkForUpdates')}
            </Button>
          }
        >
          <Button
            disabled={!update()?.available || isUpdating()}
            onClick={downloadAndInstallUpdate}
          >
            Install and restart app
          </Button>
        </Show>
      </div>
    </SettingsEntry>
  );
};

export default UpdateAppEntry;
