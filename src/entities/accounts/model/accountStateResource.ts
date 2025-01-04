import { createResource } from 'solid-js';

import { getAccountState } from '../api';

const accountStateResource = createResource(() => {
  try {
    return getAccountState();
  } catch {
    console.error("Can't get account state");
  }
});

export const getAccountStateResource = () => {
  return accountStateResource[0];
};

export const refetchAccountStateResource = () => {
  accountStateResource[1].refetch();
};
