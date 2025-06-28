import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater';
import { checkIsUpdateAvailable } from '../model';
import { createSignal } from 'solid-js';
import { LoadingBarTypeEnum, type LoadingPayload } from '@/5_entities/events';
import { getVersion } from '@tauri-apps/api/app';
import { emit } from '@tauri-apps/api/event';

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
      case 'Started': {
        contentLength = event.data.contentLength;

        emit('loading', {
          event: {
            type: LoadingBarTypeEnum.LauncherUpdate,
            version: update?.version ?? '',
            current_version: await getVersion(),
          },
          loaderUuid: '',
          fraction: 0,
          message: 'launcherUpdate.started',
        } satisfies LoadingPayload);
        break;
      }
      case 'Progress': {
        downloaded += event.data.chunkLength;
        if (contentLength) {
          emit('loading', {
            event: {
              type: LoadingBarTypeEnum.LauncherUpdate,
              version: update?.version ?? '',
              current_version: await getVersion(),
            },
            loaderUuid: '',
            fraction: downloaded / contentLength,
            message: 'launcherUpdate.progress',
          } satisfies LoadingPayload);
        }
        break;
      }
      case 'Finished': {
        emit('loading', {
          event: {
            type: LoadingBarTypeEnum.LauncherUpdate,
            version: update?.version ?? '',
            current_version: await getVersion(),
          },
          loaderUuid: '',
          fraction: null,
          message: 'launcherUpdate.finished',
        } satisfies LoadingPayload);
        break;
      }
    }
  };

  const installUpdate = async (update: Update | null) => {
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

  return { isUpdating, installUpdate };
};
