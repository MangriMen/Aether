import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import {
  isPluginDisabled,
  isPluginEnabled,
  isPluginLoading,
  type PluginState,
} from '../model';

export const usePluginStates = (state: Accessor<PluginState>) => {
  const isDisabled = createMemo(() => isPluginDisabled(state()));
  const isLoading = createMemo(() => isPluginLoading(state()));
  const isEnabled = createMemo(() => isPluginEnabled(state()));

  return { isDisabled, isLoading, isEnabled };
};
