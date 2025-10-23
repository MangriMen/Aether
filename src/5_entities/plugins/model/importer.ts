import type { ImporterCapability } from './pluginCapabilities';
import type { PluginMetadata } from './pluginManifest';

export interface Importer {
  pluginId: PluginMetadata['id'];
  capability: ImporterCapability;
}
