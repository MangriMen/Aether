import type { PluginManifest } from './plugin_manifest';

export type PluginState =
  | 'NotLoaded'
  | 'Loading'
  | 'Loaded'
  | 'Unloading'
  | 'Failed';

export interface Plugin {
  manifest: PluginManifest;
  state: PluginState;
}
