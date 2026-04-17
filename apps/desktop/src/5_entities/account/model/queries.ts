import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { commands } from '../api';
import { accountInvalidation } from './cache';
import { accountKeys } from './queryKeys';

export const useAccounts = () =>
  useQuery(() => ({
    queryKey: accountKeys.list(),
    queryFn: commands.listAccounts,
    reconcile: 'id',
    staleTime: Infinity,
  }));

export const useCreateOfflineAccount = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.createOfflineAccount,
    onSuccess: () => accountInvalidation.list(queryClient),
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
    onSuccess: () => accountInvalidation.list(queryClient),
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
    onSuccess: () => accountInvalidation.list(queryClient),
    onError: (err) => {
      showError({
        title: t('account.logoutError'),
        err,
        t,
      });
    },
  }));
};
