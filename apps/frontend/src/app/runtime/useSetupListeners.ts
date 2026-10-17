import {
  useMicaUpdateListener,
  useTransparencyUpdateListener,
  useUpdateSync,
} from '@/app/lib';
import {
  useWarningEventsListener,
  useProgressEventsListener,
} from '@/entities/events';
import { useJavaInstallationListener } from '@/entities/java';
import { useInstanceEventsListener } from '@/features/instance';
import { usePluginEventListener } from '@/features/plugins';

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
