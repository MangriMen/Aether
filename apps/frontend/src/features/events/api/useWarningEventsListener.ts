import { useEventListener, eventsEvents } from '@/platform/ipc';
import { showToast } from '@/shared/ui-kit';
import { logDebug } from '@/shared/utils';

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
