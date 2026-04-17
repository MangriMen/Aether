export const pluginKeys = {
  all: ['plugin'] as const,
  list: () => [...pluginKeys.all, 'list'] as const,
  get: (id: string) => [...pluginKeys.all, 'get', id] as const,
  enabled: (id: string) => [...pluginKeys.all, 'enabled', id] as const,
  settings: (id: string) => [...pluginKeys.all, 'settings', id] as const,
  importers: () => [...pluginKeys.list(), 'importers'] as const,
  apiVersion: () => [...pluginKeys.list(), 'api_version'] as const,
} as const;
