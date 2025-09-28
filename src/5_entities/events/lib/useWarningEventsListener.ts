import type { UnlistenFn } from '@tauri-apps/api/event';

import { onCleanup, onMount } from 'solid-js';

import { logDebug } from '@/shared/lib';
import { showToast } from '@/shared/ui';

import { listenEvent } from '../api';

export const useWarningEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const startListen = async () => {
    unlistenFn = await listenEvent('warning', (e) => {
      logDebug('[EVENT][DEBUG]', e);

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
