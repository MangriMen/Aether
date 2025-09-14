import type { PluginManifest } from './plugin_manifest';

export interface Plugin {
  enabled: boolean;
  manifest: PluginManifest;
}
