export const QUERY_KEYS = {
  PLUGIN: {
    SELF: 'plugin',
    LIST: () => [...QUERY_KEYS.PLUGIN.SELF, 'list'],
    GET: (id: string) => [...QUERY_KEYS.PLUGIN.SELF, 'get', id],
    ENABLED: (id: string) => [...QUERY_KEYS.PLUGIN.SELF, 'enabled', id],
    SETTINGS: (id: string) => [...QUERY_KEYS.PLUGIN.SELF, 'settings', id],
  },
} as const;
