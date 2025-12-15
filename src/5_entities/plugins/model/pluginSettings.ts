import type { PathMapping } from './pluginManifest';

export interface PluginSettings {
  allowedHosts: string[];
  allowedPaths: Array<PathMapping>; // (path on disk, plugin path)
}

export type EditPluginSettings = Partial<PluginSettings>;
