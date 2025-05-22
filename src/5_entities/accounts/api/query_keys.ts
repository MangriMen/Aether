export const QUERY_KEYS = {
  ACCOUNT: {
    SELF: ['account'],
    LIST: () => [...QUERY_KEYS.ACCOUNT.SELF, 'list'],
  },
} as const;
