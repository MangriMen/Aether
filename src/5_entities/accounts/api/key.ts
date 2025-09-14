export const ACCOUNT_KEY = {
  SELF: ['account'],
  LIST: () => [...ACCOUNT_KEY.SELF, 'list'],
} as const;
