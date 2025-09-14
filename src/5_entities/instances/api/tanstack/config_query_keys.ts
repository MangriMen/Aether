export const CONFIG_QUERY_KEYS = {
  IMPORT: () => [...CONFIG_QUERY_KEYS.SELF, 'import'],
  SELF: ['config'],
} as const;
