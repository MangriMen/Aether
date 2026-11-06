import type { QueryClient } from '@tanstack/solid-query';

import { pluginsCommands } from '@/platform/ipc';

import type { Plugin } from '../model/plugin';
import type { PluginId } from '../model/pluginManifest';

import { invalidateImporters } from './importerQueries';
import { pluginKeys } from './pluginKeys';

const PLUGIN_STALE_TIME = 60 * 1000;

type PluginList = Awaited<ReturnType<typeof pluginsCommands.list>>;

const pluginIdOf = (plugin: Plugin): PluginId => plugin.manifest.metadata.id;

/**
 * Lays the answer of `list` out as `detail(id)`. A detail fetched after the list request began
 * is newer than what the list says about that plugin, so it is kept.
 */
const seedDetails = (
  queryClient: QueryClient,
  plugins: PluginList,
  listStartedAt: number,
) => {
  for (const plugin of plugins) {
    const key = pluginKeys.detail(pluginIdOf(plugin));
    const current = queryClient.getQueryState(key);

    if (current?.data && current.dataUpdatedAt > listStartedAt) {
      continue;
    }

    queryClient.setQueryData(key, plugin);
  }
};

/**
 * Puts an answer of `get` into the list without making the list look fresher than it is: the
 * list keeps its own `dataUpdatedAt`. A list that arrived after the `get` began is newer, so it
 * wins; a plugin the list does not hold is not added (that is `list`'s to say).
 */
const patchList = (
  queryClient: QueryClient,
  plugin: Plugin,
  getStartedAt: number,
) => {
  const state = queryClient.getQueryState<PluginList>(pluginKeys.list());

  if (!state?.data || state.dataUpdatedAt > getStartedAt) {
    return;
  }

  queryClient.setQueryData<PluginList>(
    pluginKeys.list(),
    (list) =>
      list?.map((item) =>
        pluginIdOf(item) === pluginIdOf(plugin) ? plugin : item,
      ),
    { updatedAt: state.dataUpdatedAt },
  );
};

export const pluginsQueries = {
  // The list is the source of truth: its answer is also what every `detail(id)` is seeded from.
  list: () => ({
    queryKey: pluginKeys.list(),
    queryFn: async ({ client }: { client: QueryClient }) => {
      const startedAt = Date.now();
      const plugins = await pluginsCommands.list();

      seedDetails(client, plugins, startedAt);

      return plugins;
    },
    staleTime: PLUGIN_STALE_TIME,
  }),
  detail: (id: PluginId) => ({
    queryKey: pluginKeys.detail(id),
    queryFn: () => pluginsCommands.get(id),
    staleTime: PLUGIN_STALE_TIME,
    enabled: !!id,
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
    detail: (queryClient: QueryClient, id: PluginId) =>
      queryClient.getQueryData<Plugin>(pluginKeys.detail(id)),
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
    // plugin needs a restart. Only this plugin is asked for again; the list gets its new state.
    state: async (queryClient: QueryClient, id: PluginId) => {
      await pluginsCache.refresh.plugin(queryClient, id);
      await pluginsCache.invalidate.settings(queryClient, id);
      await pluginsCache.invalidate.importers(queryClient);
    },
    full: async (queryClient: QueryClient, id: PluginId) => {
      await pluginsCache.refresh.plugin(queryClient, id);
      await pluginsCache.invalidate.settings(queryClient, id);
      await pluginsCache.invalidate.source(queryClient, id);
      await pluginsCache.invalidate.updates(queryClient, id);
      await pluginsCache.invalidate.importers(queryClient);
    },
  },
  refresh: {
    /**
     * Asks for the plugin again and hands the answer to the list too. A plugin that cannot be
     * read any more (removed meanwhile) is for the list to settle.
     */
    plugin: async (queryClient: QueryClient, id: PluginId) => {
      const startedAt = Date.now();

      try {
        const plugin = await queryClient.fetchQuery({
          ...pluginsQueries.detail(id),
          staleTime: 0,
        });

        patchList(queryClient, plugin, startedAt);
      } catch {
        queryClient.removeQueries({ queryKey: pluginKeys.detail(id) });
        await pluginsCache.invalidate.list(queryClient);
      }
    },
  },
  remove: {
    /** The plugin is gone: nothing may serve its old state. */
    plugin: async (queryClient: QueryClient, id: PluginId) => {
      queryClient.removeQueries({ queryKey: pluginKeys.detail(id) });
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
