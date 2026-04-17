import type { QueryClient } from '@tanstack/solid-query';

import type { PluginMetadata } from '.';

import { pluginKeys } from './queryKeys';

export const invalidatePluginsData = (queryClient: QueryClient) => {
  queryClient.invalidateQueries({
    queryKey: pluginKeys.list(),
  });
};

export const invalidatePluginData = (
  queryClient: QueryClient,
  pluginId: PluginMetadata['id'],
) => {
  queryClient.invalidateQueries({
    queryKey: pluginKeys.list(),
  });
  queryClient.invalidateQueries({
    queryKey: pluginKeys.get(pluginId),
  });
  queryClient.invalidateQueries({
    queryKey: pluginKeys.enabled(pluginId),
  });
  queryClient.invalidateQueries({
    queryKey: pluginKeys.settings(pluginId),
  });
};

export const invalidateImporters = (queryClient: QueryClient) => {
  queryClient.invalidateQueries({
    queryKey: pluginKeys.importers(),
  });
};
