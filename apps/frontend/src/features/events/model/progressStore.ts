import { createStore, produce } from 'solid-js/store';

import { type ProgressEvent } from './progressEvent';
import { getFinishedIds } from './progressTree';

interface ProgressStore {
  payloads: Record<string, ProgressEvent>;
}

let progressStore: ReturnType<typeof createStore<ProgressStore>> | undefined;

const getOrCreateProgressStore = () => {
  if (!progressStore) {
    // eslint-disable-next-line solid/reactivity
    progressStore = createStore<ProgressStore>(
      {
        payloads: {},
      },
      { name: 'progressStore' },
    );
  }

  return progressStore;
};

export const useProgressStore = () => {
  return getOrCreateProgressStore();
};

/** Removes every finished root with its whole subtree. */
export const clearFinishedProgress = () => {
  const [store, setStore] = getOrCreateProgressStore();
  const ids = getFinishedIds(Object.values(store.payloads));

  setStore(
    'payloads',
    produce((p) => {
      for (const id of ids) {
        delete p[id];
      }
    }),
  );
};
