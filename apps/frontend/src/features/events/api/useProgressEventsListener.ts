import { onCleanup, onMount } from 'solid-js';
import { produce } from 'solid-js/store';

import { useEventListener, eventsCommands, eventsEvents } from '@/platform/ipc';
import { logDebug } from '@/shared/log';

import type { ProgressEvent } from '../model/progressEvent';

import { PROGRESS_EVENT_REMOVE_DELAY } from '../model/progressEvent';
import { useProgressStore } from '../model/progressStore';

export const useProgressEventsListener = () => {
  const [, setProgressStore] = useProgressStore();

  const activeTimers = new Map<string, ReturnType<typeof setTimeout>>();
  // Bars updated by live events while the initial snapshot is loading;
  // the snapshot must not overwrite them. Not needed once it is applied.
  let liveBarIds: Set<string> | undefined = new Set<string>();

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
        if (liveBarIds?.has(bar.id)) {
          continue;
        }

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
    } finally {
      liveBarIds = undefined;
    }
  };

  onMount(fetchEvents);

  useEventListener(eventsEvents.progressEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);
    liveBarIds?.add(e.payload.progressBarId);
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
