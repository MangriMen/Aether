import type { JSX } from 'solid-js';

import { useSetupListeners } from '../lib';

export type AppGlobalsProviderProps = { children?: JSX.Element };

export const AppGlobalsProvider = ({ children }: AppGlobalsProviderProps) => {
  useSetupListeners();

  return children;
};
