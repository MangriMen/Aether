import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { ACCOUNT_KEY } from './key';
import {
  changeAccountRaw,
  createOfflineAccountRaw,
  listAccountsRaw,
  logoutRaw,
} from './tauriApiRaw';
import { useTranslation } from '@/6_shared/model';
import { showError } from '@/6_shared/lib/showError';

export const useAccounts = () =>
  useQuery(() => ({
    queryKey: ACCOUNT_KEY.LIST(),
    queryFn: listAccountsRaw,
  }));

export const useCreateOfflineAccount = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: createOfflineAccountRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ACCOUNT_KEY.LIST(),
      });
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
      queryClient.invalidateQueries({
        queryKey: ACCOUNT_KEY.LIST(),
      });
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
      queryClient.invalidateQueries({
        queryKey: ACCOUNT_KEY.LIST(),
      });
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
