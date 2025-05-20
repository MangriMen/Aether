import type { UnlistenFn } from '@tauri-apps/api/event';
import { onCleanup, onMount } from 'solid-js';
import { listenEvent } from '../api';
import { isDebug } from '@/shared/model';
import { showToast } from '@/shared/ui';

export const useWarningEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const startListen = async () => {
    unlistenFn = await listenEvent('warning', (e) => {
      if (isDebug()) {
        console.log('[EVENT][DEBUG]', e);
      }

      showToast({
        title: 'Warning',
        description: e.payload.message,
        variant: 'warningFilled',
      });
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
