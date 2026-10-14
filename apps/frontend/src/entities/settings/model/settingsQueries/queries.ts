import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { settingsCommands } from '../../api';
import { settingsCache, settingsQueries } from './cache';

export const useSettings = () => useQuery(settingsQueries.get);

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
