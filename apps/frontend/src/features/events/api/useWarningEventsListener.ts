import { useEventListener, eventsEvents } from '@/platform/ipc';
import { logDebug } from '@/shared/log';
import { showToast } from '@/shared/ui-kit';

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
