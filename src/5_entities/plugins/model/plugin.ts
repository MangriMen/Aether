import type { PluginCapabilities } from './pluginCapabilities';
import type { PluginManifest } from './pluginManifest';
import type { PluginState } from './pluginState';

export interface Plugin {
  manifest: PluginManifest;
  capabilities?: PluginCapabilities;
  state: PluginState;
}
