import { makePersisted } from '@solid-primitives/storage';
import { createStore, type SetStoreFunction, type Store } from 'solid-js/store';

export interface FeatureFlagsStore {
  isAllowInstallModpacks: boolean;
}

// Simplified type definition
type FeatureFlagsStoreTuple = [
  Store<FeatureFlagsStore>,
  SetStoreFunction<FeatureFlagsStore>,
];

let featureFlagsStore: FeatureFlagsStoreTuple | undefined = undefined;

export const getOrCreateFeatureFlagsStore = (): FeatureFlagsStoreTuple => {
  if (!featureFlagsStore) {
    // Correct usage of makePersisted with createStore
    const [state, setState] = makePersisted(
      // eslint-disable-next-line solid/reactivity
      createStore<FeatureFlagsStore>({
        isAllowInstallModpacks: false,
      }),
      { name: 'feature-flags-store' }, // Better to use kebab-case or distinct storage key
    );

    featureFlagsStore = [state, setState];
  }

  return featureFlagsStore;
};

// Custom hook alias
export const useFeatureFlagsStore = () => getOrCreateFeatureFlagsStore();

// Meta-information actions
export const featureFlagsActions = {
  getBooleanFeatureFlags: (): (keyof FeatureFlagsStore)[] => [
    'isAllowInstallModpacks',
  ],
};
