import {
  useMicaUpdateListener,
  useTransparencyUpdateListener,
} from '@/features/appearance';
import {
  useWarningEventsListener,
  useProgressEventsListener,
} from '@/features/events';
import { useInstanceEventsListener } from '@/features/instance';
import { useJavaInstallationListener } from '@/features/java';
import { usePluginEventListener } from '@/features/plugins';
import { useProcessCrashListener } from '@/features/process';
import { useCacheEventListener } from '@/features/settings';
import { useUpdateSync } from '@/features/update';

export const useSetupListeners = () => {
  useInstanceEventsListener();
  useWarningEventsListener();
  useProgressEventsListener();
  usePluginEventListener();
  useProcessCrashListener();
  useJavaInstallationListener();
  useCacheEventListener();

  useMicaUpdateListener();
  useTransparencyUpdateListener();

  useUpdateSync();
};
