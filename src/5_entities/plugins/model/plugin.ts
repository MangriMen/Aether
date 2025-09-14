import type { PluginManifest } from './plugin_manifest';

export interface Plugin {
  manifest: PluginManifest;
  enabled: boolean;
}
