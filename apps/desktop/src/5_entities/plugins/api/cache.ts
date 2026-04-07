import type { QueryClient } from '@tanstack/query-core';

import type { PluginMetadata } from '../model';

import { PLUGIN_QUERY_KEYS } from './queryKeys';

export const invalidatePluginsData = (queryClient: QueryClient) => {
  queryClient.invalidateQueries({
    queryKey: PLUGIN_QUERY_KEYS.LIST(),
  });
};

export const invalidatePluginData = (
  queryClient: QueryClient,
  pluginId: PluginMetadata['id'],
) => {
  queryClient.invalidateQueries({
    queryKey: PLUGIN_QUERY_KEYS.LIST(),
  });
  queryClient.invalidateQueries({
    queryKey: PLUGIN_QUERY_KEYS.GET(pluginId),
  });
  queryClient.invalidateQueries({
    queryKey: PLUGIN_QUERY_KEYS.ENABLED(pluginId),
  });
  queryClient.invalidateQueries({
    queryKey: PLUGIN_QUERY_KEYS.SETTINGS(pluginId),
  });
};

export const invalidateImporters = (queryClient: QueryClient) => {
  queryClient.invalidateQueries({
    queryKey: PLUGIN_QUERY_KEYS.IMPORTERS(),
  });
};
