import { listenEvent } from '@/entities/events';
import { onCleanup, onMount } from 'solid-js';
import { refetchInstance } from '../model/instancesResource';
import { isDebug } from '@/shared/model';
import type { UnlistenFn } from '@tauri-apps/api/event';

export const useInstanceEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const startListen = async () => {
    unlistenFn = await listenEvent('instance', (e) => {
      if (isDebug()) {
        console.log('[EVENT][DEBUG]', e);
      }

      refetchInstance(e.payload.instancePathId);
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
