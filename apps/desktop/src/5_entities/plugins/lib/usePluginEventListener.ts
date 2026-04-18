import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { logDebug } from '@/shared/lib';

import { events } from '../api';
import {
  invalidateImporters,
  invalidatePluginData,
  invalidatePluginsData,
} from '../model';

export const usePluginEventListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await events.pluginEventDto.listen((e) => {
      logDebug('[EVENT][DEBUG]', e);

      switch (e.payload.type) {
        case 'sync':
          invalidatePluginsData(queryClient);
          invalidateImporters(queryClient);
          break;
        case 'add':
        case 'edit':
        case 'remove':
          invalidatePluginData(queryClient, e.payload.plugin_id);
          invalidateImporters(queryClient);
          break;
      }
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
