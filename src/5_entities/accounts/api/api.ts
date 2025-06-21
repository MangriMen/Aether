import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { QUERY_KEYS } from './query_keys';
import {
  changeAccountRaw,
  createOfflineAccountRaw,
  listAccountsRaw,
  logoutRaw,
} from './rawApi';
import { useTranslation } from '@/6_shared/model';
import { showError } from '@/6_shared/lib/showError';

export const useAccounts = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.ACCOUNT.LIST(),
    queryFn: listAccountsRaw,
  }));

export const useCreateOfflineAccount = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: createOfflineAccountRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.ACCOUNT.LIST() });
    },
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
    mutationFn: changeAccountRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.ACCOUNT.LIST() });
    },
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
    mutationFn: logoutRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.ACCOUNT.LIST() });
    },
    onError: (err) => {
      showError({
        title: t('account.logoutError'),
        err,
        t,
      });
    },
  }));
};
