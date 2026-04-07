import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater';

import { getVersion } from '@tauri-apps/api/app';
import { emit } from '@tauri-apps/api/event';

import { LoadingBarTypeEnum } from '@/entities/events';

export const installUpdate = async (update: Update) => {
  let contentLength: number | undefined = 0;
  let downloadedLength: number = 0;

  const handleUpdateDownloadProperties = (
    newContentLength: number | undefined,
    newDownloadedLength: number | undefined,
  ) => {
    if (newContentLength) {
      contentLength = newContentLength;
    }
    if (newDownloadedLength) {
      downloadedLength = newDownloadedLength;
    }
  };

  await update?.downloadAndInstall((event) => {
    handleUpdateDownload(
      event,
      update,
      contentLength,
      downloadedLength,
      handleUpdateDownloadProperties,
    );
  });
};

const handleUpdateDownload = async (
  event: DownloadEvent,
  update: Update,
  contentLength: number | undefined,
  downloadedLength: number | undefined,
  onUpdateDownloadProperties: (
    contentLength: number | undefined,
    downloadedLength: number | undefined,
  ) => void,
) => {
  const updateEvent = {
    type: LoadingBarTypeEnum.LauncherUpdate,
    version: update?.version ?? '',
    current_version: await getVersion(),
  } as const;

  const loaderUuid = 'launcher_update';

  switch (event.event) {
    case 'Started': {
      contentLength = event.data.contentLength;

      emit('loading', {
        event: updateEvent,
        loaderUuid,
        fraction: 0,
        message: 'launcherUpdate.started',
      });
      onUpdateDownloadProperties(contentLength, undefined);
      break;
    }
    case 'Progress': {
      const loaded = downloadedLength ?? 0 + event.data.chunkLength;
      if (contentLength) {
        emit('loading', {
          event: updateEvent,
          loaderUuid,
          fraction: loaded / contentLength,
          message: 'launcherUpdate.progress',
        });
      }
      onUpdateDownloadProperties(contentLength, loaded);
      break;
    }
    case 'Finished': {
      emit('loading', {
        event: updateEvent,
        loaderUuid,
        fraction: null,
        message: 'launcherUpdate.finished',
      });
    }
  }
};
