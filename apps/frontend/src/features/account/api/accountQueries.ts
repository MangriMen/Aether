import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError, isLauncherError } from '@/features/errors';
import { authCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

import { isAuthValidationError } from '../model/accountError';
import { accountCache, accountQueries } from './accountCache';

export const useAccounts = () => useQuery(accountQueries.list);

export const useCreateOfflineAccount = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: authCommands.createOfflineAccount,
    onSuccess: () => accountCache.invalidate.list(queryClient),
    onError: (err) => {
      if (
        isLauncherError(err) &&
        err.type === 'auth' &&
        isAuthValidationError(err.payload)
      ) {
        return;
      }

      showError({
        title: t('account.createOfflineError'),
        err,
        t,
      });
    },
  }));
};

export const useChangeAccount = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: authCommands.changeAccount,
    onSuccess: () => accountCache.invalidate.list(queryClient),
    onError: (err) => {
      showError({
        title: t('account.changeError'),
        err,
        t,
      });
    },
  }));
};

export const useLogout = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: authCommands.logout,
    onSuccess: () => accountCache.invalidate.list(queryClient),
    onError: (err) => {
      showError({
        title: t('account.logoutError'),
        err,
        t,
      });
    },
  }));
};
