export const INSTANCE_QUERY_KEYS = {
  SELF: ['instance'],
  LIST: () => [...INSTANCE_QUERY_KEYS.SELF, 'list'],
  GET: (id: string) => [...INSTANCE_QUERY_KEYS.SELF, 'get', id],
  DIR: (id: string) => [...INSTANCE_QUERY_KEYS.GET(id), 'dir'],
  PROCESS: {
    SELF: () => [...INSTANCE_QUERY_KEYS.SELF, 'process'],
    BY_INSTANCE: (id: string) => [...INSTANCE_QUERY_KEYS.PROCESS.SELF(), id],
  },
} as const;
