import type { QueryClient } from '@tanstack/query-core';

import type { PluginMetadata } from '../model';

import { PLUGIN_QUERY_KEYS } from './query_keys';

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
};
