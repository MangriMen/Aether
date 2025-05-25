import type { PathMapping } from './plugin_manifest';

export interface PluginSettings {
  allowed_hosts: string[];
  allowed_paths: Array<PathMapping>; // (path on disk, plugin path)
}
