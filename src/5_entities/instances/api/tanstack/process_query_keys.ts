export const PROCESS_QUERY_KEYS = {
  SELF: ['process'],
  LIST: () => [...PROCESS_QUERY_KEYS.SELF, 'list'],
  BY_INSTANCE: (id: string) => [...PROCESS_QUERY_KEYS.SELF, 'instance', id],
} as const;
