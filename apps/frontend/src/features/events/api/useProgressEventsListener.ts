import { onMount } from 'solid-js';

import { useEventListener, eventsCommands, eventsEvents } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';
import { logDebug, logError } from '@/shared/log';
import { showToast } from '@/shared/ui-kit';

import type { ProgressEvent } from '../model/progressEvent';

import { useProgressStore } from '../model/progressStore';

export const useProgressEventsListener = () => {
  const [progressStore, setProgressStore] = useProgressStore();
  const [{ t }] = useTranslation();

  // Bars whose failure was already reported, so the toast comes up once.
  const notifiedFailures = new Set<string>();

  // Bars updated by live events while the initial snapshot is loading;
  // the snapshot must not overwrite them. Not needed once it is applied.
  let liveBarIds: Set<string> | undefined = new Set<string>();

  // Bars are never removed on their own: the user clears finished ones.
  const addEvent = (payload: ProgressEvent) => {
    const current = progressStore.payloads[payload.progressBarId];

    if (
      current !== undefined &&
      current.status.type !== 'running' &&
      payload.status.type === 'running'
    ) {
      // Only reported: the core sends a bar's events in order.
      logError('[EVENT] running event after the end of its bar', {
        current,
        payload,
      });
    }

    setProgressStore('payloads', payload.progressBarId, payload);
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
  });
};
