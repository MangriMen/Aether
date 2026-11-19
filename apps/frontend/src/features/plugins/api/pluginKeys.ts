export const pluginKeys = {
  all: ['plugin'] as const,
  list: () => [...pluginKeys.all, 'list'] as const,
  detail: (id: string) => [...pluginKeys.all, 'detail', id] as const,
  skippedDirs: () => [...pluginKeys.all, 'skipped-dirs'] as const,
  settings: (id: string) => [...pluginKeys.all, 'settings', id] as const,
  apiVersion: () => [...pluginKeys.list(), 'apiVersion'] as const,
  readme: (id: string, version: string) =>
    [...pluginKeys.all, 'readme', id, version] as const,
  source: (id: string) => [...pluginKeys.all, 'source', id] as const,
  updates: (id: string) => [...pluginKeys.all, 'updates', id] as const,
  /** What release `tag` asks for against the installed version; all of a plugin without `tag`. */
  updatePreview: (id: string, tag?: string) =>
    tag === undefined
      ? ([...pluginKeys.all, 'update-preview', id] as const)
      : ([...pluginKeys.all, 'update-preview', id, tag] as const),
  releases: (provider: string, identifier: string) =>
    [...pluginKeys.all, 'releases', provider, identifier] as const,
  preview: (provider: string, identifier: string, tag: string) =>
    [...pluginKeys.all, 'preview', provider, identifier, tag] as const,
  /** Whether the core finds an `allowedHosts` entry wide; a pure function of the text. */
  wideHost: (entry: string) => [...pluginKeys.all, 'wide-host', entry] as const,
  providers: () => [...pluginKeys.all, 'providers'] as const,
  commandApprovals: () => [...pluginKeys.all, 'command-approvals'] as const,
  importers: () => [...pluginKeys.all, 'importer'] as const,
  importerList: () => [...pluginKeys.importers(), 'list'] as const,
  /** Mutation key of every restart: the plugin is the mutation's variable. */
  reloadAll: () => [...pluginKeys.all, 'reload'] as const,
  /** Mutation key of every update: the plugin is in the mutation's variables. */
  updateAll: () => [...pluginKeys.all, 'update'] as const,
} as const;
