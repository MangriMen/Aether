export const PROCESS_QUERY_KEYS = {
  BY_INSTANCE: (id: string) => [...PROCESS_QUERY_KEYS.SELF, 'instance', id],
  LIST: () => [...PROCESS_QUERY_KEYS.SELF, 'list'],
  SELF: ['process'],
} as const;
