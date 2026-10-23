import { useQueryClient } from '@tanstack/solid-query';

import { eventsEvents, useEventListener } from '@/platform/ipc';
import { logDebug, logError } from '@/shared/log';

import { settingsCache } from './settingsCache';

export const useCacheEventListener = () => {
  const queryClient = useQueryClient();

  useEventListener(eventsEvents.cacheEventDto, async (e) => {
    logDebug('[EVENT][DEBUG]', e);

    try {
      switch (e.payload.type) {
        case 'changed':
          await settingsCache.invalidate.cacheUsage(queryClient);
          break;
        case 'busyChanged':
          await settingsCache.setCacheBusy(queryClient, e.payload.busy);
          break;
      }
    } catch (error) {
      logError('Failed to apply the cache event:', error);
    }
  });
};
