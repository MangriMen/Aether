import type { ProcessPayload } from '@/entities/events';
import {
  isProcessPayload,
  listenEvent,
  ProcessPayloadType,
} from '@/entities/events';
import { isDebug } from '@/shared/model';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMount } from 'solid-js';

export const useHideOnLaunch = () => {
  const listenProcessEvents = () => {
    listenEvent<ProcessPayload>('process', async (e) => {
      if (isDebug()) {
        console.log('[EVENT][DEBUG]', e);
      }

      if (isProcessPayload(e.payload)) {
        if (e.payload.event === ProcessPayloadType.Launched) {
          await getCurrentWindow().hide();
        } else if (e.payload.event === ProcessPayloadType.Finished) {
          await getCurrentWindow().show();
        }
      }
    });
  };

  onMount(listenProcessEvents);
};
