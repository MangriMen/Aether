import { createSignal } from 'solid-js';

import { getAccounts } from '../api';

import type { Account } from './account';

const [accountsResource, setAccountsResource] = createSignal<Account[]>([]);

const fetchAccounts = async () => {
  try {
    setAccountsResource(await getAccounts());
  } catch (e) {
    console.error("Can't get account state:", e);
  }
};

export const initializeAccountsResource = fetchAccounts;

export const useAccounts = () => accountsResource;

export const refetchAccounts = fetchAccounts;
