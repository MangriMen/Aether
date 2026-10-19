import { type Accessor, createMemo, createRoot } from 'solid-js';
import { createStore, produce } from 'solid-js/store';

import { type ProgressEvent } from './progressEvent';
import {
  buildProgressIndex,
  getFinishedIds,
  type ProgressIndex,
} from './progressTree';

interface ProgressStore {
  payloads: Record<string, ProgressEvent>;
}

let progressStore: ReturnType<typeof createStore<ProgressStore>> | undefined;
let progressIndex: Accessor<ProgressIndex> | undefined;

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

/**
 * The tree of the bars in the store. Rebuilt when a bar comes, goes or changes its parent or
 * mirror, not when one only moves on: those events reach the card of the bar alone.
 */
export const useProgressIndex = (): Accessor<ProgressIndex> => {
  if (!progressIndex) {
    const [store] = getOrCreateProgressStore();
    progressIndex = createRoot(() =>
      createMemo(() => buildProgressIndex(Object.values(store.payloads))),
    );
  }

  return progressIndex;
};

/** Removes every finished root with its whole subtree. */
export const clearFinishedProgress = () => {
  const [, setStore] = getOrCreateProgressStore();
  const ids = getFinishedIds(useProgressIndex()());

  setStore(
    'payloads',
    produce((p) => {
      for (const id of ids) {
        delete p[id];
      }
    }),
  );
};
