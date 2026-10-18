import { onCleanup, onMount } from 'solid-js';
import { produce } from 'solid-js/store';

import { useEventListener, eventsCommands, eventsEvents } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';
import { logDebug } from '@/shared/log';
import { showToast } from '@/shared/ui-kit';

import type { ProgressEvent } from '../model/progressEvent';

import {
  getProgressRemoveDelay,
  isProgressFailed,
} from '../model/progressEvent';
import { useProgressStore } from '../model/progressStore';
import { getDescendantIds } from '../model/progressTree';

export const useProgressEventsListener = () => {
  const [progressStore, setProgressStore] = useProgressStore();
  const [{ t }] = useTranslation();

  // Bars whose failure was already reported, so the toast comes up once.
  const notifiedFailures = new Set<string>();

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

  // Removing a root takes its whole subtree with it, so no orphans stay behind.
  const removeEvent = (id: ProgressEvent['progressBarId']) => {
    const ids = [
      id,
      ...getDescendantIds(Object.values(progressStore.payloads), id),
    ];

    for (const barId of ids) {
      clearTimeout(activeTimers.get(barId));
      activeTimers.delete(barId);
    }

    setProgressStore(
      'payloads',
      produce((p) => {
        for (const barId of ids) {
          delete p[barId];
        }
      }),
    );
  };

  const removeEventDelayed = (payload: ProgressEvent) => {
    const timer = setTimeout(() => {
      removeEvent(payload.progressBarId);
    }, getProgressRemoveDelay(payload));

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
          stage: bar.stage,
          status: bar.status,
          parentId: bar.parentId,
          mirrorOf: bar.mirrorOf,
          bytesDone: bar.bytesDone,
          bytesTotal: bar.bytesTotal,
          speed: bar.speed,
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

    // A mirror fails with the shared download it shows, which reports it once.
    if (
      e.payload.status.type === 'failed' &&
      e.payload.mirrorOf === null &&
      !notifiedFailures.has(e.payload.progressBarId)
    ) {
      notifiedFailures.add(e.payload.progressBarId);
      showToast({
        title: t('events.installFailed'),
        description: e.payload.status.message,
        variant: 'destructive',
      });
    }

    // Only roots expire; children go with their root.
    if (
      e.payload.parentId === null &&
      (e.payload.fraction === null || isProgressFailed(e.payload))
    ) {
      removeEventDelayed(e.payload);
    }
  });

  onCleanup(() => {
    if (activeTimers.size !== 0) {
      activeTimers.forEach(clearTimeout);
    }
  });
};
