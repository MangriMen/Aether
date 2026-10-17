import { onCleanup, onMount } from 'solid-js';
import { produce } from 'solid-js/store';

import { useEventListener, eventsCommands, eventsEvents } from '@/platform/ipc';
import { logDebug } from '@/shared/lib';

import type { ProgressEvent } from '../model/progressEvent';

import { useProgressStore } from '../model/progressStore';

export const PROGRESS_EVENT_REMOVE_DELAY = 5000;

export const useProgressEventsListener = () => {
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
    }, PROGRESS_EVENT_REMOVE_DELAY);

    activeTimers.set(payload.progressBarId, timer);
  };

  const fetchEvents = async () => {
    try {
      const bars = await eventsCommands.listProgressBars();

      for (const bar of bars) {
        addEvent({
          event: bar.progressType,
          progressBarId: bar.id,
          message: bar.message,
          fraction:
            bar.current === null || bar.total === null
              ? 0
              : bar.current / bar.total,
        });
      }
    } catch {
      /* empty */
    }
  };

  onMount(fetchEvents);

  useEventListener(eventsEvents.progressEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);
    addEvent(e.payload);

    if (e.payload.fraction === null) {
      removeEventDelayed(e.payload);
    }
  });

  onCleanup(() => {
    if (activeTimers.size !== 0) {
      activeTimers.forEach(clearTimeout);
    }
  });
};
