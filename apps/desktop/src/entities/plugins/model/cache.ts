import type { QueryClient } from '@tanstack/solid-query';

import type { PluginId } from './pluginManifest';

import { commands } from '../api';
import { pluginKeys } from './queryKeys';

export const pluginsQueries = {
  list: () => ({
    queryKey: pluginKeys.list(),
    queryFn: commands.list,
    staleTime: 60 * 1000,
  }),
  get: (queryClient: QueryClient, id: PluginId) => ({
    queryKey: pluginKeys.get(id),
    queryFn: () => commands.get(id),
    enabled: !!id,
    initialData: () => pluginsCache.getData.fromList(queryClient, id),
    staleTime: 60 * 1000,
  }),
  settings: (id: PluginId) => ({
    queryKey: pluginKeys.settings(id),
    queryFn: () => commands.getSettings(id),
    enabled: !!id,
  }),
  apiVersion: () => ({
    queryKey: pluginKeys.apiVersion(),
    queryFn: commands.getApiVersion,
  }),
} as const;

export const pluginsCache = {
  getData: {
    list: (queryClient: QueryClient) =>
      queryClient.getQueryData<Awaited<ReturnType<typeof commands.list>>>(
        pluginKeys.list(),
      ),
    fromList: (queryClient: QueryClient, id: PluginId) =>
      pluginsCache.getData
        .list(queryClient)
        ?.find((p) => p.manifest.metadata.id === id),
    settings: (queryClient: QueryClient, id: PluginId) =>
      queryClient.getQueryData<
        Awaited<ReturnType<typeof commands.getSettings>>
      >(pluginKeys.settings(id)),
  },
  invalidate: {
    all: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: pluginKeys.all,
      }),
    list: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: pluginKeys.list(),
      }),
    get: (queryClient: QueryClient, id: PluginId) =>
      queryClient.invalidateQueries({
        queryKey: pluginKeys.get(id),
      }),
    settings: (queryClient: QueryClient, id: PluginId) =>
      queryClient.invalidateQueries({
        queryKey: pluginKeys.settings(id),
      }),
    full: async (queryClient: QueryClient, id: PluginId) => {
      await pluginsCache.invalidate.list(queryClient);
      await pluginsCache.invalidate.get(queryClient, id);
      await pluginsCache.invalidate.settings(queryClient, id);
    },
  },
  prefetch: {
    settings: (queryClient: QueryClient, id: PluginId) =>
      queryClient.prefetchQuery(pluginsQueries.settings(id)),
  },
};
