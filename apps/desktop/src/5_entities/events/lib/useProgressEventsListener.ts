import { type UnlistenFn } from '@tauri-apps/api/event';
import { onCleanup, onMount } from 'solid-js';
import { produce } from 'solid-js/store';

import { logDebug } from '@/shared/lib';
import { useProgressStore } from '@/widgets/app-titlebar';

import type { ProgressEvent } from '../model';

import { commands, events } from '../api';

export const useProgressEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const [, setProgressStore] = useProgressStore();

  const timers = new Set<number>();

  const fetchEvents = async () => {
    try {
      const bars = await commands.listProgressBars();

      for (const bar of bars) {
        addEvent({
          event: bar.progressType,
          progressBarId: bar.id,
          message: bar.message,
          fraction: bar.current / bar.total,
        });
      }
    } catch {
      /* empty */
    }
  };

  const addEvent = (payload: ProgressEvent) => {
    setProgressStore('payloads', (payloads) => ({
      ...payloads,
      [payload.progressBarId]: payload,
    }));
  };

  const removeEvent = (payload: ProgressEvent) => {
    setProgressStore(
      'payloads',
      produce((payloads) => delete payloads[payload.progressBarId]),
    );
  };

  const delayedRemoveEvent = (payload: ProgressEvent) => {
    timers.add(
      setTimeout(() => {
        removeEvent(payload);
      }, 1000),
    );
  };

  const startListen = async () => {
    fetchEvents();

    unlistenFn = await events.progressEventDto.listen((e) => {
      logDebug('[EVENT][DEBUG]', e);

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
