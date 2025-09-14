export const QUERY_KEYS = {
  SETTINGS: {
    GET: () => [...QUERY_KEYS.SETTINGS.SELF, 'get'],
    RAM: () => [...QUERY_KEYS.SETTINGS.SELF, 'ram'],
    SELF: ['settings'],
  },
} as const;
