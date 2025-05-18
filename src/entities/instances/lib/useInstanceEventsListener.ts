import { listenEvent } from '@/entities/events/@x/instances';
import { onCleanup, onMount } from 'solid-js';
import { isDebug } from '@/shared/model';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { useQueryClient } from '@tanstack/solid-query';
import { invalidateInstanceData } from '../api';

export const useInstanceEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await listenEvent('instance', (e) => {
      if (isDebug()) {
        console.log('[EVENT][DEBUG]', e);
      }

      invalidateInstanceData(queryClient, e.payload.instanceId);
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
