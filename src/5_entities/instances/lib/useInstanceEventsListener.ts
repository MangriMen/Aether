import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { listenEvent } from '@/entities/events/@x/instances';
import { debugLog } from '@/shared/lib/log';

import { invalidateInstanceData } from '../api';

export const useInstanceEventsListener = () => {
  let unlistenFn: undefined | UnlistenFn = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await listenEvent('instance', (e) => {
      debugLog('[EVENT][DEBUG]', e);

      invalidateInstanceData(queryClient, e.payload.instanceId);
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
