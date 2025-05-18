import { useWarningEventsListener } from '@/entities/events';
import { useInstanceEventsListener } from '@/entities/instances';

export const useSetupListeners = () => {
  useInstanceEventsListener();
  useWarningEventsListener();
};
