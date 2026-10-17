import {
  useMicaUpdateListener,
  useTransparencyUpdateListener,
} from '@/app/lib';
import { useJavaInstallationListener } from '@/entities/java';
import {
  useWarningEventsListener,
  useProgressEventsListener,
} from '@/features/events';
import { useInstanceEventsListener } from '@/features/instance';
import { usePluginEventListener } from '@/features/plugins';
import { useUpdateSync } from '@/features/update';

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
