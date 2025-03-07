import { useMaximizeObserver } from '@/shared/lib';
import { usePreventRightClick } from './usePreventRightClick';
import { useInstanceEventsListener } from '@/entities/instances';
import { useWarningEventsListener } from '@/entities/events';

export const useSetup = () => {
  usePreventRightClick();
  useMaximizeObserver();

  // Event listeners
  useInstanceEventsListener();
  useWarningEventsListener();
};
