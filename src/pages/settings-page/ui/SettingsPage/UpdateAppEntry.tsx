import { relaunch } from '@tauri-apps/plugin-process';
import { Component, ComponentProps, createMemo } from 'solid-js';

import { Button } from '@/shared/ui';

import { updateResource } from '@/entities/update';

import { SettingsEntry } from '../SettingsEntry';

export type UpdateAppEntryProps = ComponentProps<'div'>;

const UpdateAppEntry: Component<UpdateAppEntryProps> = (props) => {
  const [update, { refetch }] = updateResource;

  const checkUpdates = () => {
    refetch();
  };

  const downloadAndInstallUpdate = async () => {
    if (!update()?.available) {
      return;
    }

    let downloaded: number = 0;
    let contentLength: number | undefined = 0;

    update()?.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength;
          console.log(`started downloading ${event.data.contentLength} bytes`);
          break;
        case 'Progress':
          downloaded += event.data.chunkLength;
          console.log(`downloaded ${downloaded} from ${contentLength}`);
          break;
        case 'Finished':
          console.log('download finished');
          break;
      }
    });

    await relaunch();
  };

  const description = createMemo(() => {
    if (update()?.available) {
      return (
        <div class='flex flex-col gap-2'>
          <span>Version: {update()?.version}</span>
          <span>Release date: {update()?.date}</span>
        </div>
      );
    }

    return 'There is no updates';
  });

  return (
    <SettingsEntry title='Check updates' description={description()} {...props}>
      <div class='flex flex-col gap-2'>
        <Button onClick={checkUpdates} loading={update.loading}>
          Check for updates
        </Button>
        <Button
          disabled={!update()?.available}
          onClick={downloadAndInstallUpdate}
        >
          Install and restart app
        </Button>
      </div>
    </SettingsEntry>
  );
};

export default UpdateAppEntry;
