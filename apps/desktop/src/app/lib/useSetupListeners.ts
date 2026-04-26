import {
  useWarningEventsListener,
  useProgressEventsListener,
} from '@/entities/events';
import { useInstanceEventsListener } from '@/entities/instances';
import { usePluginEventListener } from '@/entities/plugins';

import { useMicaUpdateListener } from './useMicaUpdateListener';
import { useTransparencyUpdateListener } from './useTransparencyUpdateListener';
import { useUpdateSync } from './useUpdateSync';

export const useSetupListeners = () => {
  useInstanceEventsListener();
  useWarningEventsListener();
  useProgressEventsListener();
  usePluginEventListener();

  useMicaUpdateListener();
  useTransparencyUpdateListener();

  useUpdateSync();
};
