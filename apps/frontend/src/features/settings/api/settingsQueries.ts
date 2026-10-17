import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/features/errors';
import { settingsCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';
import { showToast } from '@/shared/ui-kit';

import { settingsCache, settingsQueries } from './settingsCache';

export const useSettings = () => useQuery(settingsQueries.get);

/** How much disk the launcher folders take, by what they hold. */
export const useStorageUsage = () => useQuery(settingsQueries.storageUsage);

/** How much each cache takes, and whether clearing it is refused now. */
export const useCacheUsage = () => useQuery(settingsQueries.cacheUsage);

export const useMaxRam = () => useQuery(settingsQueries.maxRam);

/**
 * Choose (or, with `null`, drop the choice of) the provider serving a platform.
 * The edit command carries the rest of the settings too, so they are sent back as they are —
 * read just before the write, as the cached copy may be older than another edit.
 */
export const useSetPlatformProvider = () => {
  const edit = useEditSettings();

  return useMutation(() => ({
    mutationFn: async (platformProvider: {
      platform: string;
      providerId: string | null;
    }) => {
      const current = await settingsCommands.get();

      return edit.mutateAsync({
        maxConcurrentDownloads: current.maxConcurrentDownloads,
        platformProvider,
      });
    },
  }));
};

/** Empty the caches the app can fetch again (content metadata, icons, modpack archives). */
export const useClearCache = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: settingsCommands.clearCache,
    onSuccess: (outcome) => {
      if (outcome === 'busy') {
        showToast({ title: t('settings.storage.busyToast') });
      }

      // What the lists showed from the cache is fetched again.
      void queryClient.invalidateQueries();
    },
    onError: (err) => {
      showError({ title: t('settings.storage.clearError'), err, t });
    },
  }));
};

export const useEditSettings = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: settingsCommands.edit,
    onSuccess: (data) => {
      settingsCache.set(queryClient, data);
    },
    onError: (err) => {
      showError({
        title: t('settings.changeError'),
        err,
        t,
      });
    },
  }));
};
