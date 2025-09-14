import type { UnlistenFn } from '@tauri-apps/api/event';

import { onCleanup, onMount } from 'solid-js';
import { produce } from 'solid-js/store';

import { debugLog } from '@/shared/lib/log';
import { useProgressStore } from '@/widgets/app-titlebar';

import type { LoadingPayload } from '../model';

import { getLoadingBars, listenEvent } from '../api';

export const useProgressEventsListener = () => {
  let unlistenFn: undefined | UnlistenFn = undefined;

  const [, setProgressStore] = useProgressStore();

  const timers = new Set<number>();

  const fetchEvents = async () => {
    try {
      const bars = await getLoadingBars();

      for (const bar of Object.values(bars)) {
        addEvent({
          event: bar.barType,
          fraction: bar.current / bar.total,
          loaderUuid: bar.loadingBarUuid,
          message: bar.message,
        });
      }
    } catch {
      /* empty */
    }
  };

  const addEvent = (payload: LoadingPayload) => {
    setProgressStore('payloads', (payloads) => ({
      ...payloads,
      [payload.loaderUuid]: payload,
    }));
  };

  const removeEvent = (payload: LoadingPayload) => {
    setProgressStore(
      'payloads',
      produce((payloads) => delete payloads[payload.loaderUuid]),
    );
  };

  const delayedRemoveEvent = (payload: LoadingPayload) => {
    timers.add(
      setTimeout(() => {
        removeEvent(payload);
      }, 1000),
    );
  };

  const startListen = async () => {
    fetchEvents();

    unlistenFn = await listenEvent('loading', (e) => {
      debugLog('[EVENT][DEBUG]', e);

      if (e.payload.fraction === null) {
        delayedRemoveEvent(e.payload);
        return;
      }

      addEvent(e.payload);
    });
  };

  const cleanUp = () => {
    for (const timer of timers) {
      clearTimeout(timer);
    }
    unlistenFn?.();
  };

  onMount(startListen);
  onCleanup(cleanUp);
};
