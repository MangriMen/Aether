import type { QueryClient } from '@tanstack/solid-query';

import type { PluginId } from './pluginManifest';

import { pluginsCommands } from '../api';
import { pluginKeys } from './queryKeys';

export const pluginsQueries = {
  list: () => ({
    queryKey: pluginKeys.list(),
    queryFn: pluginsCommands.list,
    staleTime: 60 * 1000,
  }),
  get: (queryClient: QueryClient, id: PluginId) => ({
    queryKey: pluginKeys.get(id),
    queryFn: () => pluginsCommands.get(id),
    enabled: !!id,
    initialData: () => pluginsCache.getData.fromList(queryClient, id),
    staleTime: 60 * 1000,
  }),
  settings: (id: PluginId) => ({
    queryKey: pluginKeys.settings(id),
    queryFn: () => pluginsCommands.getSettings(id),
    enabled: !!id,
  }),
  apiVersion: () => ({
    queryKey: pluginKeys.apiVersion(),
    queryFn: pluginsCommands.getApiVersion,
  }),
  source: (id: PluginId) => ({
    queryKey: pluginKeys.source(id),
    queryFn: () => pluginsCommands.getPluginSource(id),
    enabled: !!id,
  }),
  updates: (id: PluginId) => ({
    queryKey: pluginKeys.updates(id),
    queryFn: () => pluginsCommands.checkForUpdates(id),
    enabled: !!id,
  }),
  preview: (url: string) => ({
    queryKey: pluginKeys.preview(url),
    queryFn: () => pluginsCommands.previewPluginFromProvider('git_hub', url),
    enabled: url.length > 0,
  }),
} as const;

export const pluginsCache = {
  getData: {
    list: (queryClient: QueryClient) =>
      queryClient.getQueryData<
        Awaited<ReturnType<typeof pluginsCommands.list>>
      >(pluginKeys.list()),
    fromList: (queryClient: QueryClient, id: PluginId) =>
      pluginsCache.getData
        .list(queryClient)
        ?.find((p) => p.manifest.metadata.id === id),
    settings: (queryClient: QueryClient, id: PluginId) =>
      queryClient.getQueryData<
        Awaited<ReturnType<typeof pluginsCommands.getSettings>>
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
    source: (queryClient: QueryClient, id: PluginId) =>
      queryClient.invalidateQueries({
        queryKey: pluginKeys.source(id),
      }),
    updates: (queryClient: QueryClient, id: PluginId) =>
      queryClient.invalidateQueries({
        queryKey: pluginKeys.updates(id),
      }),
    full: async (queryClient: QueryClient, id: PluginId) => {
      await pluginsCache.invalidate.list(queryClient);
      await pluginsCache.invalidate.get(queryClient, id);
      await pluginsCache.invalidate.settings(queryClient, id);
      await pluginsCache.invalidate.source(queryClient, id);
      await pluginsCache.invalidate.updates(queryClient, id);
    },
  },
  prefetch: {
    settings: (queryClient: QueryClient, id: PluginId) =>
      queryClient.prefetchQuery(pluginsQueries.settings(id)),
  },
};
