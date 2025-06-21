import { createMemo, type Accessor } from 'solid-js';
import type { Account } from '../model';

export const useSortedAccounts = (
  accounts: Accessor<Account[]>,
  sortBy?: Accessor<keyof Omit<Account, 'id'>>,
) => {
  const sortedAccounts = createMemo(() => sortAccounts(accounts(), sortBy?.()));

  return sortedAccounts;
};

const sortAccounts = (
  accounts: Account[],
  sortBy: keyof Omit<Account, 'id'> = 'active',
) =>
  [...accounts].sort((a, b) => {
    const aValue = a[sortBy];
    const bValue = b[sortBy];

    if (aValue == bValue) {
      return 0;
    }

    return aValue ? -1 : 1;
  });
