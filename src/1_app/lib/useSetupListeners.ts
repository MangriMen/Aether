import {
  useWarningEventsListener,
  useProgressEventsListener,
} from '@/entities/events';
import { useInstanceEventsListener } from '@/entities/instances';
import { usePluginEventListener } from '@/entities/plugins';

import { useMicaUpdateListener } from './useMicaUpdateListener';

export const useSetupListeners = () => {
  useInstanceEventsListener();
  useWarningEventsListener();
  useProgressEventsListener();
  usePluginEventListener();

  useMicaUpdateListener();
};
