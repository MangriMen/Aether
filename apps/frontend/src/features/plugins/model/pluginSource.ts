import type {
  PluginProviderDto,
  PluginSourceTypeDto,
} from '@/platform/bindings/plugin';

/** Where a plugin can be installed from; what the backend has a provider for. */
export type PluginSourceType = PluginSourceTypeDto;

/** A provider the backend offers, with the name it gives itself. */
export type PluginProvider = PluginProviderDto;
