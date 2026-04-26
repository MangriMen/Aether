import { createStore } from 'solid-js/store';

export interface UpdateStore {
  isUpdating: boolean;
}

let updateStore: ReturnType<typeof createStore<UpdateStore>> | undefined;

export const getOrCreateUpdateStore = () => {
  if (!updateStore) {
    // eslint-disable-next-line solid/reactivity
    updateStore = createStore<UpdateStore>(
      {
        isUpdating: false,
      },
      { name: 'updateStore' },
    );
  }
  return updateStore;
};

export const useUpdateStore = () => {
  return getOrCreateUpdateStore();
};
