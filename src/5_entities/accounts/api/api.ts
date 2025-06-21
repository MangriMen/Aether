import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { QUERY_KEYS } from './query_keys';
import {
  changeAccountRaw,
  createOfflineAccountRaw,
  listAccountsRaw,
  logoutRaw,
} from './rawApi';
import {
  getTranslatedError,
  isLauncherError,
  useTranslation,
} from '@/6_shared/model';
import { showToast } from '@/6_shared/ui';

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
      if (isLauncherError(err)) {
        showToast({
          title: t('account.createOfflineError'),
          description: getTranslatedError(err, t),
          variant: 'destructive',
        });
      }
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
      if (isLauncherError(err)) {
        showToast({
          title: t('account.changeError'),
          description: getTranslatedError(err, t),
          variant: 'destructive',
        });
      }
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
      if (isLauncherError(err)) {
        showToast({
          title: t('account.logoutError'),
          description: getTranslatedError(err, t),
          variant: 'destructive',
        });
      }
    },
  }));
};
