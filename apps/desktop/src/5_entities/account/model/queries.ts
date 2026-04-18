import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { commands } from '../api';
import { accountCache, accountQueries } from './cache';

export const useAccounts = () => useQuery(accountQueries.list);

export const useCreateOfflineAccount = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.createOfflineAccount,
    onSuccess: () => accountCache.invalidate.list(queryClient),
    onError: (err) => {
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
    mutationFn: commands.changeAccount,
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
    mutationFn: commands.logout,
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
