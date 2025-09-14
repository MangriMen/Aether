export const QUERY_KEYS = {
  SETTINGS: {
    SELF: ['settings'],
    GET: () => [...QUERY_KEYS.SETTINGS.SELF, 'get'],
    RAM: () => [...QUERY_KEYS.SETTINGS.SELF, 'ram'],
  },
} as const;
