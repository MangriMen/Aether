import type { PathMapping } from './plugin_manifest';

export interface PluginSettings {
  allowedHosts: string[];
  allowedPaths: Array<PathMapping>; // (path on disk, plugin path)
}

export type EditPluginSettings = Partial<PluginSettings>;
