import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { isLauncherError, useTranslation } from '@/shared/model';

import { authCommands } from '../api';
import { accountCache, accountQueries } from './cache';
import { isAuthValidationError } from './error';

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
