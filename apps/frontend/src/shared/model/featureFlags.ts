import type { SetStoreFunction, Store } from 'solid-js/store';

import { createStore } from 'solid-js/store';

export interface FeatureFlagsStore {
  isAllowInstallModpacks: boolean;
}

let featureFlagsStore:
  | ReturnType<typeof createStore<FeatureFlagsStore>>
  | undefined = undefined;

export const getOrCreateFeatureFlagsStore = (): [
  Store<FeatureFlagsStore>,
  SetStoreFunction<FeatureFlagsStore>,
] => {
  if (!featureFlagsStore) {
    // eslint-disable-next-line solid/reactivity
    featureFlagsStore = createStore<FeatureFlagsStore>(
      {
        isAllowInstallModpacks: false,
      },
      { name: 'featureFlagsStore' },
    );
  }

  return featureFlagsStore;
};

export const useFeatureFlagsStore = () => {
  return getOrCreateFeatureFlagsStore();
};

export const featureFlagsActions = {
  getBooleanFeatureFlags: (): (keyof FeatureFlagsStore)[] => [
    'isAllowInstallModpacks',
  ],
};
