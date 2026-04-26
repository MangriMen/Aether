import { useMutation, useQuery } from '@tanstack/solid-query';

import { useTranslation } from '@/shared/model';
import { showToast } from '@/shared/ui';

import { commands } from '../api';
import { updateKeys } from './queryKeys';

export const useCheckUpdate = () =>
  useQuery(() => ({
    queryKey: updateKeys.check(),
    queryFn: commands.checkForUpdates,
  }));

export const useInstallUpdate = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.installUpdate,
    onError: () => {
      showToast({
        title: t('update.updateError'),
        variant: 'destructive',
      });
    },
  }));
};
