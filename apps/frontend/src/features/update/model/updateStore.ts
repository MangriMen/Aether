import { createStore } from 'solid-js/store';

interface UpdateStore {
  isUpdating: boolean;
}

let updateStore: ReturnType<typeof createStore<UpdateStore>> | undefined;

const getOrCreateUpdateStore = () => {
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
