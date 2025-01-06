import { createResource } from 'solid-js';

import { getAccounts } from '../api';

import { Account } from './account';

const accountStateResource = createResource<Account[]>(
  () => {
    try {
      return getAccounts();
    } catch {
      console.error("Can't get account state");
      return [];
    }
  },
  { initialValue: [] },
);

export const getAccountStateResource = () => {
  return accountStateResource[0];
};

export const refetchAccountStateResource = () => {
  accountStateResource[1].refetch();
};
