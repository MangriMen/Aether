import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { listenEvent } from '@/entities/events/@x/instances';
import { logDebug } from '@/shared/lib';

import { invalidateInstanceData } from '../api';

export const useInstanceEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await listenEvent('instance', (e) => {
      logDebug('[EVENT][DEBUG]', e);

      invalidateInstanceData(queryClient, e.payload.instanceId);
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
