import {
  useProgressEventsListener,
  useWarningEventsListener,
} from '@/entities/events';
import { useInstanceEventsListener } from '@/entities/instances';

import { useMicaUpdateListener } from './useMicaUpdateListener';

export const useSetupListeners = () => {
  useInstanceEventsListener();
  useWarningEventsListener();
  useProgressEventsListener();

  useMicaUpdateListener();
};
