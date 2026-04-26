import type { PluginDtoState } from '../api';

export type PluginState = PluginDtoState;

export const isPluginDisabled = (state: PluginState) =>
  state === 'NotLoaded' || state === 'Failed';

export const isPluginLoading = (state: PluginState) =>
  state === 'Loading' || state === 'Unloading';

export const isPluginEnabled = (state: PluginState) => state === 'Loaded';
