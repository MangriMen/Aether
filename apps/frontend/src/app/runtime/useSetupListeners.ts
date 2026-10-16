import { useMicaUpdateListener } from '@/app/lib/useMicaUpdateListener';
import { useTransparencyUpdateListener } from '@/app/lib/useTransparencyUpdateListener';
import { useUpdateSync } from '@/app/lib/useUpdateSync';
import {
  useWarningEventsListener,
  useProgressEventsListener,
} from '@/entities/events';
import { useInstanceEventsListener } from '@/entities/instances';
import { useJavaInstallationListener } from '@/entities/java';
import { usePluginEventListener } from '@/entities/plugins';

export const useSetupListeners = () => {
  useInstanceEventsListener();
  useWarningEventsListener();
  useProgressEventsListener();
  usePluginEventListener();
  useJavaInstallationListener();

  useMicaUpdateListener();
  useTransparencyUpdateListener();

  useUpdateSync();
};
