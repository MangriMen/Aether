import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { QUERY_KEYS } from './query_keys';
import {
  changeAccountRaw,
  createOfflineAccountRaw,
  listAccountsRaw,
  logoutRaw,
} from './rawApi';

export const useAccounts = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.ACCOUNT.LIST(),
    queryFn: listAccountsRaw,
  }));

export const useCreateOfflineAccount = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: createOfflineAccountRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.ACCOUNT.LIST() });
    },
  }));
};

export const useChangeAccount = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: changeAccountRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.ACCOUNT.LIST() });
    },
  }));
};

export const useLogout = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: logoutRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.ACCOUNT.LIST() });
    },
  }));
};
