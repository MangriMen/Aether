import { batch, onCleanup, onMount } from 'solid-js';
import { produce } from 'solid-js/store';

import { useEventListener, eventsCommands, eventsEvents } from '@/platform/ipc';
import { logDebug, logError } from '@/shared/log';

import type { ProgressEvent } from '../model/progressEvent';

import { useProgressStore } from '../model/progressStore';

const FLUSH_DELAY_MS = 16;

export const useProgressEventsListener = () => {
  // A failed bar is shown by the panel; the toast of the failure is the initiator's
  // (`showError`), so none comes from here.
  const [progressStore, setProgressStore] = useProgressStore();

  // Bars updated by live events while the initial snapshot is loading;
  // the snapshot must not overwrite them. Not needed once it is applied.
  let liveBarIds: Set<string> | undefined = new Set<string>();

  // Bars that have ended, kept after the user clears them. The webview can hand over events it
  // already delivered again, long after the bar ended; a running one must not bring it back.
  const endedBarIds = new Set<string>();

  // Bars are never removed on their own: the user clears finished ones.
  const addEvent = (payload: ProgressEvent) => {
    const id = payload.progressBarId;

    if (payload.status.type !== 'running') {
      endedBarIds.add(id);
    } else if (endedBarIds.has(id)) {
      logError('[EVENT] running event after the end of its bar', payload);
      return;
    }

    // A mirror is cancelled with the call-off of the download it shows, or of its own install.
    // The install of a waiting bar goes on and takes the download up itself, under a new mirror,
    // so a cancelled one is no part of what the user sees.
    if (payload.mirrorOf !== null && payload.status.type === 'cancelled') {
      setProgressStore(
        'payloads',
        produce((payloads) => {
          delete payloads[id];
        }),
      );
      return;
    }

    // A bar that goes on running keeps its status object, so what reads the status of the
    // bars (the failure of a subtree, say) is not woken by every step of every download.
    if (
      payload.status.type === 'running' &&
      progressStore.payloads[id]?.status.type === 'running'
    ) {
      const { status: _, ...step } = payload;
      setProgressStore('payloads', id, step);
      return;
    }

    setProgressStore('payloads', id, payload);
  };

  // Events come in bursts, one message each; they are applied together a frame later, in the
  // order they came, so the tree is drawn once per frame and not once per event.
  let pending: ProgressEvent[] = [];
  let flushTimer: ReturnType<typeof setTimeout> | undefined;

  const flush = () => {
    flushTimer = undefined;
    const events = pending;
    pending = [];
    batch(() => events.forEach(addEvent));
  };

  onCleanup(() => clearTimeout(flushTimer));

  const enqueueEvent = (payload: ProgressEvent) => {
    pending.push(payload);
    flushTimer ??= setTimeout(flush, FLUSH_DELAY_MS);
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
          stage: bar.stage,
          status: bar.status,
          parentId: bar.parentId,
          mirrorOf: bar.mirrorOf,
          bytesDone: bar.bytesDone,
          bytesTotal: bar.bytesTotal,
          speed: bar.speed,
          cancel: bar.cancel,
        });
      }
    } catch (error) {
      logError('Failed to list progress bars:', error);
    } finally {
      liveBarIds = undefined;
    }
  };

  onMount(fetchEvents);

  useEventListener(eventsEvents.progressEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);
    liveBarIds?.add(e.payload.progressBarId);
    enqueueEvent(e.payload);
  });
};
