import type { PluginManifest } from './plugin_manifest';
import type { PluginState } from './plugin_state';

export interface Plugin {
  manifest: PluginManifest;
  state: PluginState;
}
