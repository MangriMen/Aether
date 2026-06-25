import type {
  PluginCapabilitiesDto,
  PluginContentProviderCapabilityDto,
  PluginImporterCapabilityDto,
  PluginUpdaterCapabilityDto,
} from '../api';

export type PluginCapabilities = PluginCapabilitiesDto;
export type ImporterCapability = PluginImporterCapabilityDto;
export type UpdaterCapability = PluginUpdaterCapabilityDto;
export type ContentProviderCapability = PluginContentProviderCapabilityDto;
