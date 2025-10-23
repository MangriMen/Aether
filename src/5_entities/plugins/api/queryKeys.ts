export const PLUGIN_QUERY_KEYS = {
  SELF: 'plugin',
  LIST: () => [...PLUGIN_QUERY_KEYS.SELF, 'list'],
  GET: (id: string) => [...PLUGIN_QUERY_KEYS.SELF, 'get', id],
  ENABLED: (id: string) => [...PLUGIN_QUERY_KEYS.SELF, 'enabled', id],
  SETTINGS: (id: string) => [...PLUGIN_QUERY_KEYS.SELF, 'settings', id],
  IMPORTERS: () => [...PLUGIN_QUERY_KEYS.LIST(), 'importers'],
} as const;
