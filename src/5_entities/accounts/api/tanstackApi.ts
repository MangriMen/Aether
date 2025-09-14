import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import { ACCOUNT_KEY } from './key';
import {
  changeAccountRaw,
  createOfflineAccountRaw,
  listAccountsRaw,
  logoutRaw,
} from './tauriApiRaw';

export const useAccounts = () =>
  useQuery(() => ({
    queryFn: listAccountsRaw,
    queryKey: ACCOUNT_KEY.LIST(),
    reconcile: 'id',
  }));

export const useCreateOfflineAccount = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: createOfflineAccountRaw,
    onError: (err) => {
      showError({
        err,
        t,
        title: t('account.createOfflineError'),
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ACCOUNT_KEY.LIST(),
      });
    },
  }));
};

export const useChangeAccount = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: changeAccountRaw,
    onError: (err) => {
      showError({
        err,
        t,
        title: t('account.changeError'),
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ACCOUNT_KEY.LIST(),
      });
    },
  }));
};

export const useLogout = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: logoutRaw,
    onError: (err) => {
      showError({
        err,
        t,
        title: t('account.logoutError'),
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ACCOUNT_KEY.LIST(),
      });
    },
  }));
};
