export const SETTINGS_QUERY_KEYS = {
  SELF: ['settings'],
  GET: () => [...SETTINGS_QUERY_KEYS.SELF, 'get'],
  RAM: () => [...SETTINGS_QUERY_KEYS.SELF, 'ram'],
} as const;
