import { useQueryClient } from '@tanstack/solid-query';

import { instanceEvents, useEventListener } from '@/platform/ipc';
import { logDebug } from '@/shared/log';

import {
  invalidateInstanceContent,
  invalidateInstanceData,
} from './instanceCache';

export const useInstanceEventsListener = () => {
  const queryClient = useQueryClient();

  useEventListener(instanceEvents.instanceEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    if (e.payload.event === 'content_synced') {
      // Only the content's metadata and icons changed.
      invalidateInstanceContent(queryClient, e.payload.instanceId);
      return;
    }

    invalidateInstanceData(queryClient, e.payload.instanceId);
  });
};
