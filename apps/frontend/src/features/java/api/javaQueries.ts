import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError, isLauncherError } from '@/features/errors';
import { javaCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';
import { logDebug } from '@/shared/utils';

import { javaCache } from './javaCache';
import { javaKeys } from './javaQueryKeys';

export const useJavaList = () =>
  useQuery(() => ({
    queryKey: javaKeys.list(),
    queryFn: javaCommands.list,
    reconcile: 'majorVersion',
  }));

export const useInstallJava = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: javaCommands.install,
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
    mutationFn: javaCommands.testJre,
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
    mutationFn: javaCommands.edit,
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
    mutationFn: javaCommands.remove,
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

export const useDiscoverJava = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: javaCommands.discover,
    onError: (err) => {
      showError({
        title: t('java.removeError'),
        err,
        t,
      });
    },
  }));
};

export const useActiveJavaInstallations = () =>
  useQuery(() => ({
    queryKey: javaKeys.getActiveInstallations(),
    queryFn: javaCommands.getActiveInstallations,
  }));
