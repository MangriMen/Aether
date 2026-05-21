import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { logDebug, showError } from '@/shared/lib';
import { isLauncherError, useTranslation } from '@/shared/model';

import { commands } from '../api';
import { javaCache } from './cache';
import { javaKeys } from './queryKeys';

export const useJavaList = () =>
  useQuery(() => ({
    queryKey: javaKeys.list(),
    queryFn: commands.list,
    reconcile: 'majorVersion',
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

export const useTestJre = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.testJre,
    onError: (err) => {
      showError({
        title: t('java.testError'),
        err,
        t,
      });
    },
  }));
};

export const useEditJava = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.edit,
    onSuccess: () => {
      javaCache.invalidate.list(queryClient);
    },
    onError: (err) => {
      showError({
        title: t('java.editError'),
        err,
        t,
      });
    },
  }));
};

export const useRemoveJava = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.remove,
    onSuccess: () => {
      javaCache.invalidate.list(queryClient);
    },
    onError: (err) => {
      if (
        isLauncherError(err) &&
        err.type === 'java' &&
        err.payload.code === 'NOT_FOUND'
      ) {
        logDebug(err);
        return;
      }

      showError({
        title: t('java.removeError'),
        err,
        t,
      });
    },
  }));
};
