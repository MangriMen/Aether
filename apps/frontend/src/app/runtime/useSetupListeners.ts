import {
  useMicaUpdateListener,
  useTransparencyUpdateListener,
  useUpdateSync,
} from '@/app/lib';
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
