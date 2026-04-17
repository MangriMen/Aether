import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import {
  editDefaultInstanceSettingsRaw,
  getDefaultInstanceSettingsRaw,
} from '../../api';
import { defaultInstanceSettingsQueryKeys } from './queryKeys';

export const useDefaultInstanceSettings = () =>
  useQuery(() => ({
    queryKey: defaultInstanceSettingsQueryKeys.get(),
    queryFn: getDefaultInstanceSettingsRaw,
  }));

export const useEditDefaultInstanceSettings = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: editDefaultInstanceSettingsRaw,
    onSuccess: (data) => {
      queryClient.setQueryData(
        defaultInstanceSettingsQueryKeys.get(),
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
