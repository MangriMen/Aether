import { type UnlistenFn } from '@tauri-apps/api/event';
import { onCleanup, onMount } from 'solid-js';
import { produce } from 'solid-js/store';

import { useProgressStore } from '@/widgets/app-titlebar';

import type { ProgressEvent } from '../model';

import { logDebug } from '../../../shared/lib';
import { commands, events } from '../api';

export const useProgressEventsListener = () => {
  let unlistenFn: UnlistenFn | undefined = undefined;

  const [, setProgressStore] = useProgressStore();

  const activeTimers = new Map<string, number>();

  const addEvent = (payload: ProgressEvent) => {
    if (activeTimers.has(payload.progressBarId)) {
      clearTimeout(activeTimers.get(payload.progressBarId));
      activeTimers.delete(payload.progressBarId);
    }

    setProgressStore('payloads', payload.progressBarId, payload);
  };

  const removeEvent = (id: ProgressEvent['progressBarId']) => {
    setProgressStore(
      'payloads',
      produce((p) => delete p[id]),
    );
    activeTimers.delete(id);
  };

  const removeEventDelayed = (payload: ProgressEvent) => {
    const timer = setTimeout(() => {
      removeEvent(payload.progressBarId);
    }, 1000);

    activeTimers.set(payload.progressBarId, timer);
  };

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

  const startListen = async () => {
    fetchEvents();

    unlistenFn = await events.progressEventDto.listen((e) => {
      logDebug('[EVENT][DEBUG]', e);
      addEvent(e.payload);

      if (e.payload.fraction === null) {
        removeEventDelayed(e.payload);
      }
    });
  };

  const cleanUp = () => {
    if (activeTimers.size !== 0) {
      activeTimers.forEach(clearTimeout);
    }
    unlistenFn?.();
  };

  onMount(startListen);
  onCleanup(cleanUp);
};
