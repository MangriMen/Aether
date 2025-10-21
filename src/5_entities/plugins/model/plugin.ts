import type { PluginManifest } from './pluginManifest';
import type { PluginState } from './pluginState';

export interface Plugin {
  manifest: PluginManifest;
  state: PluginState;
}
