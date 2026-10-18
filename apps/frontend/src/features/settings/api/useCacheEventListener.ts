import { useQueryClient } from '@tanstack/solid-query';

import { eventsEvents, useEventListener } from '@/platform/ipc';
import { logDebug } from '@/shared/log';

import { settingsCache } from './settingsCache';

export const useCacheEventListener = () => {
  const queryClient = useQueryClient();

  useEventListener(eventsEvents.cacheEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    settingsCache.invalidate.cacheUsage(queryClient);
  });
};
