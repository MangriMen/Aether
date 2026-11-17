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
import {
  useAllowRefusedHost,
  usePluginEventListener,
} from '@/features/plugins';
import {
  usePostExitHookFailedListener,
  useProcessCrashListener,
} from '@/features/process';
import { useCacheEventListener } from '@/features/settings';
import { useUpdateSync } from '@/features/update';

export const useSetupListeners = () => {
  useInstanceEventsListener();
  useWarningEventsListener();
  useProgressEventsListener();
  usePluginEventListener();
  // Error toasts of a refused host offer to allow it.
  useAllowRefusedHost();
  useProcessCrashListener();
  usePostExitHookFailedListener();
  useJavaInstallationListener();
  useCacheEventListener();

  useMicaUpdateListener();
  useTransparencyUpdateListener();

  useUpdateSync();
};
