import { onMount, type JSX } from 'solid-js';

import { getOrCreateFeatureFlagsStore } from '@/shared/model';

import { useSetupListeners } from './useSetupListeners';

export type AppGlobalsProviderProps = { children?: JSX.Element };

export const AppGlobalsProvider = ({ children }: AppGlobalsProviderProps) => {
  useSetupListeners();

  onMount(() => {
    getOrCreateFeatureFlagsStore();
  });

  return children;
};
