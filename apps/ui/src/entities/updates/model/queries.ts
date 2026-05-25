import { useMutation, useQuery } from '@tanstack/solid-query';

import { useTranslation } from '@/shared/model';
import { showToast } from '@/shared/ui';

import { updateCommands } from '../api';
import { updateKeys } from './queryKeys';

export const useCheckUpdate = () =>
  useQuery(() => ({
    queryKey: updateKeys.check(),
    queryFn: updateCommands.checkForUpdates,
  }));

export const useInstallUpdate = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: updateCommands.installUpdate,
    onError: () => {
      showToast({
        title: t('update.updateError'),
        variant: 'destructive',
      });
    },
  }));
};
