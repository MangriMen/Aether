import type { UnlistenFn } from '@tauri-apps/api/event';

import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup, onMount } from 'solid-js';

import { events, PROGRESS_EVENT_REMOVE_DELAY } from '@/entities/events';

import { javaCache, useActiveJavaInstallations } from '../model';

export const useJavaInstallationListener = () => {
  const queryClient = useQueryClient();
  const activeInstallations = useActiveJavaInstallations();

  let unlistenFn: UnlistenFn | undefined | null = undefined;
  let removeTimeoutId: number | undefined = undefined;

  const startListen = () => {
    events.progressEventDto
      .listen((e) => {
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
      })
      .then((unlisten) => {
        if (unlistenFn === null) {
          unlisten();
        } else {
          unlistenFn = unlisten;
        }
      });
  };

  const cleanUp = () => {
    if (!unlistenFn) {
      unlistenFn = null;
    } else {
      unlistenFn();
    }

    clearTimeout(removeTimeoutId);
  };

  onMount(startListen);
  onCleanup(cleanUp);
};
