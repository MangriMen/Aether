import { createMemo, type Accessor } from 'solid-js';

import type { Account } from '../model';

export const useSortedAccounts = (accounts: Accessor<Account[]>) => {
  const sortedAccounts = createMemo(() => sortAccounts(accounts()));
  return sortedAccounts;
};

const sortAccounts = (accounts: Account[]) => {
  return [...accounts].sort((a, b) => {
    if (a.active !== b.active) {
      return a.active ? -1 : 1;
    }

    return a.username.localeCompare(b.username);
  });
};
