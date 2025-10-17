import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { listenEvent } from '@/entities/events';
import { logDebug } from '@/shared/lib';

import { invalidatePluginData } from '../api';

export const usePluginEventListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await listenEvent('plugin', (e) => {
      logDebug('[EVENT][DEBUG]', e);

      invalidatePluginData(queryClient, e.payload.pluginId);
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
