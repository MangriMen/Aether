import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { listenEvent, PluginEventTypeEnum } from '@/entities/events';
import { logDebug } from '@/shared/lib';

import { invalidatePluginData, invalidatePluginsData } from '../api';

export const usePluginEventListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const queryClient = useQueryClient();

  const startListen = async () => {
    unlistenFn = await listenEvent('plugin', (e) => {
      logDebug('[EVENT][DEBUG]', e);

      switch (e.payload.event.type) {
        case PluginEventTypeEnum.Sync:
          invalidatePluginsData(queryClient);
          break;
        case PluginEventTypeEnum.Add:
        case PluginEventTypeEnum.Edit:
        case PluginEventTypeEnum.Remove:
          invalidatePluginData(queryClient, e.payload.event.plugin_id);
          break;
      }
    });
  };

  const stopListen = () => unlistenFn?.();

  onMount(startListen);
  onCleanup(stopListen);
};
