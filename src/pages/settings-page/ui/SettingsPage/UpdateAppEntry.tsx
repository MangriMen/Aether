import MdiLoadingIcon from '@iconify/icons-mdi/loading';
import { Icon } from '@iconify-icon/solid';
import { emit } from '@tauri-apps/api/event';
import { relaunch } from '@tauri-apps/plugin-process';
import type { DownloadEvent } from '@tauri-apps/plugin-updater';
import type { Component, ComponentProps } from 'solid-js';
import { createSignal, Show } from 'solid-js';

import { Button, showToast } from '@/shared/ui';

import type { LoadingPayload } from '@/entities/minecraft';
import { LoadingBarTypeEnum } from '@/entities/minecraft';
import { updateResource } from '@/entities/update';

import { getVersion } from '@tauri-apps/api/app';

import { SettingsEntry } from '../SettingsEntry';

export type UpdateAppEntryProps = ComponentProps<'div'>;

const UpdateAppEntry: Component<UpdateAppEntryProps> = (props) => {
  const [update, { refetch }] = updateResource;

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
      title='Check updates'
      description={
        <Show
          when={!update.loading}
          fallback={<Icon class='animate-spin' icon={MdiLoadingIcon} />}
        >
          <Show when={update()?.available} fallback='There is no updates'>
            <div class='flex flex-col'>
              <span>
                An updated version of the application is now available!
              </span>
              <span>It will restart automatically after installation.</span>
              <span>Version: {update()?.version}</span>
              <Show when={update()?.date}>
                <span>Release date: {update()?.date}</span>
              </Show>
            </div>
          </Show>
        </Show>
      }
      {...props}
    >
      <div class='flex flex-col gap-2'>
        <Button
          loading={update.loading}
          disabled={isUpdating()}
          onClick={checkUpdates}
        >
          Check for updates
        </Button>
        <Button
          disabled={!update()?.available || isUpdating()}
          onClick={downloadAndInstallUpdate}
        >
          Install and restart app
        </Button>
      </div>
    </SettingsEntry>
  );
};

export default UpdateAppEntry;
