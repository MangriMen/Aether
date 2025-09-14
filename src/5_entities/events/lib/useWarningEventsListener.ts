import type { UnlistenFn } from '@tauri-apps/api/event';

import { onCleanup, onMount } from 'solid-js';

import { debugLog } from '@/shared/lib/log';
import { showToast } from '@/shared/ui';

import { listenEvent } from '../api';

export const useWarningEventsListener = () => {
  let unlistenFn: undefined | UnlistenFn = undefined;

  const startListen = async () => {
    unlistenFn = await listenEvent('warning', (e) => {
      debugLog('[EVENT][DEBUG]', e);

      showToast({
        description: e.payload.message,
        title: 'Warning',
        variant: 'warningFilled',
      });
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
