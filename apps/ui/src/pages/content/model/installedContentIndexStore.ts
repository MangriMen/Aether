import type { SetStoreFunction, Store } from 'solid-js/store';

import { createStore } from 'solid-js/store';

export type InstalledContentIndexStore = {
  content: Record<string, Record<string, string | undefined>>;
};

export const createInstalledContentIndexStore = (): [
  Store<InstalledContentIndexStore>,
  SetStoreFunction<InstalledContentIndexStore>,
] => {
  const [store, setStore] = createStore<InstalledContentIndexStore>({
    content: {},
  });

  return [store, setStore];
};
