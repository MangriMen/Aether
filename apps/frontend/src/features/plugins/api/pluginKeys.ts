export const pluginKeys = {
  all: ['plugin'] as const,
  list: () => [...pluginKeys.all, 'list'] as const,
  settings: (id: string) => [...pluginKeys.all, 'settings', id] as const,
  apiVersion: () => [...pluginKeys.list(), 'api_version'] as const,
  source: (id: string) => [...pluginKeys.all, 'source', id] as const,
  updates: (id: string) => [...pluginKeys.all, 'updates', id] as const,
  preview: (identifier: string) =>
    [...pluginKeys.all, 'preview', identifier] as const,
  importers: () => [...pluginKeys.all, 'importer'] as const,
  importerList: () => [...pluginKeys.importers(), 'list'] as const,
  /** Mutation key of every restart: the plugin is the mutation's variable. */
  reloadAll: () => [...pluginKeys.all, 'reload'] as const,
} as const;
