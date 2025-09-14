import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater';

import { getVersion } from '@tauri-apps/api/app';
import { emit } from '@tauri-apps/api/event';
import { createSignal } from 'solid-js';

import { LoadingBarTypeEnum, type LoadingPayload } from '@/entities/events';

import { checkIsUpdateAvailable } from '../model';

export const useInstallUpdate = () => {
  const [isUpdating, setIsUpdating] = createSignal(false);

  let downloaded: number = 0;
  let contentLength: number | undefined = 0;

  const resetCounters = () => {
    downloaded = 0;
    contentLength = 0;
  };

  const handleUpdateDownload = async (event: DownloadEvent, update: Update) => {
    switch (event.event) {
      case 'Finished': {
        emit('loading', {
          event: {
            current_version: await getVersion(),
            type: LoadingBarTypeEnum.LauncherUpdate,
            version: update?.version ?? '',
          },
          fraction: null,
          loaderUuid: '',
          message: 'launcherUpdate.finished',
        } satisfies LoadingPayload);
        break;
      }
      case 'Progress': {
        downloaded += event.data.chunkLength;
        if (contentLength) {
          emit('loading', {
            event: {
              current_version: await getVersion(),
              type: LoadingBarTypeEnum.LauncherUpdate,
              version: update?.version ?? '',
            },
            fraction: downloaded / contentLength,
            loaderUuid: '',
            message: 'launcherUpdate.progress',
          } satisfies LoadingPayload);
        }
        break;
      }
      case 'Started': {
        contentLength = event.data.contentLength;

        emit('loading', {
          event: {
            current_version: await getVersion(),
            type: LoadingBarTypeEnum.LauncherUpdate,
            version: update?.version ?? '',
          },
          fraction: 0,
          loaderUuid: '',
          message: 'launcherUpdate.started',
        } satisfies LoadingPayload);
        break;
      }
    }
  };

  const installUpdate = async (update: null | Update) => {
    if (!checkIsUpdateAvailable(update)) {
      return;
    }

    resetCounters();

    setIsUpdating(true);
    try {
      await update?.downloadAndInstall((event) =>
        handleUpdateDownload(event, update),
      );
    } catch {
      /* empty */
    }
    setIsUpdating(false);
  };

  return { installUpdate, isUpdating };
};
