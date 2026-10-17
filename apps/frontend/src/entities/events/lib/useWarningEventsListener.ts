import { useEventListener } from '@/platform/ipc';
import { logDebug } from '@/shared/lib';
import { showToast } from '@/shared/ui';

import { eventsEvents } from '../api';

export const useWarningEventsListener = () => {
  useEventListener(eventsEvents.warningEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    showToast({
      title: 'Warning',
      description: e.payload.message,
      variant: 'warning',
    });
  });
};
