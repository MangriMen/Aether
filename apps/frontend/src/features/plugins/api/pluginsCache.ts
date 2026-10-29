import type { QueryClient } from '@tanstack/solid-query';

import { pluginsCommands } from '@/platform/ipc';

import type { PluginId } from '../model/pluginManifest';

import { invalidateImporters } from './importerQueries';
import { pluginKeys } from './pluginKeys';

export const pluginsQueries = {
  list: () => ({
    queryKey: pluginKeys.list(),
    queryFn: pluginsCommands.list,
    staleTime: 60 * 1000,
  }),
  skippedDirs: () => ({
    queryKey: pluginKeys.skippedDirs(),
    queryFn: pluginsCommands.listSkippedDirs,
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
    all: async (queryClient: QueryClient) => {
      await queryClient.invalidateQueries({
        queryKey: pluginKeys.all,
      });
    },
    // Importers are registered by the plugins that are running, so the list changes
    // with a plugin's state.
    importers: invalidateImporters,
    list: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: pluginKeys.list(),
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
    // The plugin's state and what it is computed from: its settings decide whether a running
    // plugin needs a restart.
    state: async (queryClient: QueryClient, id: PluginId) => {
      await pluginsCache.invalidate.list(queryClient);
      await pluginsCache.invalidate.settings(queryClient, id);
      await pluginsCache.invalidate.importers(queryClient);
    },
    full: async (queryClient: QueryClient, id: PluginId) => {
      await pluginsCache.invalidate.list(queryClient);
      await pluginsCache.invalidate.settings(queryClient, id);
      await pluginsCache.invalidate.source(queryClient, id);
      await pluginsCache.invalidate.updates(queryClient, id);
      await pluginsCache.invalidate.importers(queryClient);
    },
  },
  prefetch: {
    settings: (queryClient: QueryClient, id: PluginId) =>
      queryClient.prefetchQuery(pluginsQueries.settings(id)),
  },
};
