import { processEvents, useReadyEventListener } from '@/platform/ipc';
import { logDebug } from '@/shared/log';

import type { ProcessEvent } from '../model/processEvent';

/** Settles once the listener is registered, see `useReadyEventListener`. */
export const useReadyProcessEventListener = (
  callback: (event: ProcessEvent) => void,
) =>
  useReadyEventListener(processEvents.processEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    callback(e.payload);
  });

export const useProcessEventListener = (
  callback: (event: ProcessEvent) => void,
): void => {
  void useReadyProcessEventListener(callback);
};
