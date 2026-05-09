import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { commands } from '../api';
import { javaCache } from './cache';
import { javaKeys } from './queryKeys';

export const useJavaList = () =>
  useQuery(() => ({
    queryKey: javaKeys.list(),
    queryFn: commands.list,
  }));

export const useInstallJava = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.install,
    onSuccess: () => javaCache.invalidate.list(queryClient),
    onError: (err) => {
      showError({
        title: t('java.installError'),
        err,
        t,
      });
    },
  }));
};

export const useTestJava = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.test,
    onError: (err) => {
      showError({
        title: t('java.testError'),
        err,
        t,
      });
    },
  }));
};
