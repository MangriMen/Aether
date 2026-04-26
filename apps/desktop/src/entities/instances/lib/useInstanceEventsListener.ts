import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { logDebug } from '../../../shared/lib';
import { events } from '../api';
import { invalidateInstanceData } from '../model';

export const useInstanceEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await events.instanceEventDto.listen((e) => {
      logDebug('[EVENT][DEBUG]', e);

      invalidateInstanceData(queryClient, e.payload.instanceId);
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
