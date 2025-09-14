export const CONFIG_QUERY_KEYS = {
  SELF: ['config'],
  IMPORT: () => [...CONFIG_QUERY_KEYS.SELF, 'import'],
} as const;
