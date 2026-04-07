import { createStore } from 'solid-js/store';

import { type LoadingPayload } from '@/entities/events';

export interface ProgressStore {
  payloads: Record<string, LoadingPayload>;
}

let progressStore: ReturnType<typeof createStore<ProgressStore>> | undefined;

export const getOrCreateProgressStore = () => {
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
