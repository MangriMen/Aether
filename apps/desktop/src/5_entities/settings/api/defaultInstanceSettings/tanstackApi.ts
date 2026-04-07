import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import { DEFAULT_INSTANCE_SETTINGS_QUERY_KEYS } from './defaultInstanceSettingsQueryKeys';
import {
  editDefaultInstanceSettingsRaw,
  getDefaultInstanceSettingsRaw,
} from './tauriApi';

export const useDefaultInstanceSettings = () =>
  useQuery(() => ({
    queryKey: DEFAULT_INSTANCE_SETTINGS_QUERY_KEYS.GET(),
    queryFn: getDefaultInstanceSettingsRaw,
  }));

export const useEditDefaultInstanceSettings = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: editDefaultInstanceSettingsRaw,
    onSuccess: (data) => {
      queryClient.setQueryData(
        DEFAULT_INSTANCE_SETTINGS_QUERY_KEYS.GET(),
        () => data,
      );
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
