export const QUERY_KEYS = {
  PLUGIN: {
    ENABLED: (id: string) => [...QUERY_KEYS.PLUGIN.SELF, 'enabled', id],
    GET: (id: string) => [...QUERY_KEYS.PLUGIN.SELF, 'get', id],
    LIST: () => [...QUERY_KEYS.PLUGIN.SELF, 'list'],
    SELF: 'plugin',
    SETTINGS: (id: string) => [...QUERY_KEYS.PLUGIN.SELF, 'settings', id],
  },
} as const;
