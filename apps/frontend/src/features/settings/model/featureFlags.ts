import { makePersisted } from '@solid-primitives/storage';
import { createStore, type SetStoreFunction, type Store } from 'solid-js/store';

// Narrow to a union of flag names when the first flag is added.
export type FeatureFlagsStore = Partial<Record<string, boolean>>;

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
      createStore<FeatureFlagsStore>({}),
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
  getBooleanFeatureFlags: (): readonly string[] => [],
};
