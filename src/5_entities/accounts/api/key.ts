export const ACCOUNT_KEY = {
  LIST: () => [...ACCOUNT_KEY.SELF, 'list'],
  SELF: ['account'],
} as const;
