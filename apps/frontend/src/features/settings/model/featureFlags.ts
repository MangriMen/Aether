import { makePersisted } from '@solid-primitives/storage';
import { createStore, type SetStoreFunction, type Store } from 'solid-js/store';
import * as v from 'valibot';

// Narrow to a union of flag names when the first flag is added.
export type FeatureFlagsStore = Partial<Record<string, boolean>>;

// Simplified type definition
type FeatureFlagsStoreTuple = [
  Store<FeatureFlagsStore>,
  SetStoreFunction<FeatureFlagsStore>,
];

const FeatureFlagsSchema = v.record(v.string(), v.boolean());

// A hand-edited or outdated value must not reach the app as flags it cannot read.
export const deserializeFeatureFlags = (raw: string): FeatureFlagsStore => {
  try {
    const result = v.safeParse(FeatureFlagsSchema, JSON.parse(raw));

    if (result.success) {
      return result.output;
    }
  } catch {
    // not JSON
  }

  return {};
};

let featureFlagsStore: FeatureFlagsStoreTuple | undefined = undefined;

export const getOrCreateFeatureFlagsStore = (): FeatureFlagsStoreTuple => {
  if (!featureFlagsStore) {
    // Correct usage of makePersisted with createStore
    const [state, setState] = makePersisted(
      createStore<FeatureFlagsStore>({}),
      { name: 'feature-flags-store', deserialize: deserializeFeatureFlags },
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
