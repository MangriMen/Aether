import type {
  ImporterCapability,
  UpdaterCapability,
} from './pluginCapabilities';
import type { PluginMetadata } from './pluginManifest';

export interface CapabilityEntry<C> {
  pluginId: PluginMetadata['id'];
  capability: C;
}

export type ImporterCapabilityEntry = CapabilityEntry<ImporterCapability>;
export type UpdaterCapabilityEntry = CapabilityEntry<UpdaterCapability>;
