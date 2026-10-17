import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup } from 'solid-js';

import {
  eventsEvents,
  PROGRESS_EVENT_REMOVE_DELAY,
} from '@/entities/events/@x/java';
import { useEventListener } from '@/platform/ipc';

import { javaCache, useActiveJavaInstallations } from '../model';

export const useJavaInstallationListener = () => {
  const queryClient = useQueryClient();
  const activeInstallations = useActiveJavaInstallations();

  let removeTimeoutId: number | undefined = undefined;

  useEventListener(eventsEvents.progressEventDto, (e) => {
    if (e.payload.event.type !== 'java_download') {
      return;
    }

    const version = e.payload.event.version;

    const currentActive = activeInstallations.data ?? [];
    const isAlreadyTracked = currentActive.includes(version);

    if (!isAlreadyTracked) {
      javaCache.invalidate.getActiveInstallations(queryClient);
    }

    if (e.payload.fraction === null) {
      clearTimeout(removeTimeoutId);

      removeTimeoutId = window.setTimeout(() => {
        javaCache.invalidate.list(queryClient);
        javaCache.invalidate.getActiveInstallations(queryClient);
      }, PROGRESS_EVENT_REMOVE_DELAY);
    }
  });

  onCleanup(() => clearTimeout(removeTimeoutId));
};
