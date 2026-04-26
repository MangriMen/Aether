import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { logDebug } from '@/shared/lib';

import { events } from '../api';
import { pluginsCache } from '../model';

export const usePluginEventListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await events.pluginEventDto.listen((e) => {
      logDebug('[EVENT][DEBUG]', e);

      switch (e.payload.type) {
        case 'sync':
          pluginsCache.invalidate.all(queryClient);
          break;
        case 'add':
        case 'edit':
        case 'remove':
          pluginsCache.invalidate.full(queryClient, e.payload.plugin_id);
          break;
      }
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
