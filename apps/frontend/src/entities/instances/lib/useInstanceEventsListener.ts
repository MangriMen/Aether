import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { logDebug } from '@/shared/lib';

import { instanceEvents } from '../api';
import { invalidateInstanceContent, invalidateInstanceData } from '../model';

export const useInstanceEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await instanceEvents.instanceEventDto.listen((e) => {
      logDebug('[EVENT][DEBUG]', e);

      if (e.payload.event === 'content_synced') {
        // Only the content's metadata and icons changed.
        invalidateInstanceContent(queryClient, e.payload.instanceId);
        return;
      }

      invalidateInstanceData(queryClient, e.payload.instanceId);
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
