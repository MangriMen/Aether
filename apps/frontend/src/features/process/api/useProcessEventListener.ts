import { processEvents, useEventListener } from '@/platform/ipc';
import { logDebug } from '@/shared/utils';

import type { ProcessEvent } from '../model/processEvent';

export const useProcessEventListener = (
  callback: (event: ProcessEvent) => void,
) => {
  useEventListener(processEvents.processEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    callback(e.payload);
  });
};
