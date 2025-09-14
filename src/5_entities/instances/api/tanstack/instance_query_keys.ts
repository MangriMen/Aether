export const INSTANCE_QUERY_KEYS = {
  DIR: (id: string) => [...INSTANCE_QUERY_KEYS.GET(id), 'dir'],
  GET: (id: string) => [...INSTANCE_QUERY_KEYS.SELF, 'get', id],
  LIST: () => [...INSTANCE_QUERY_KEYS.SELF, 'list'],
  PROCESS: {
    BY_INSTANCE: (id: string) => [...INSTANCE_QUERY_KEYS.PROCESS.SELF(), id],
    SELF: () => [...INSTANCE_QUERY_KEYS.SELF, 'process'],
  },
  SELF: ['instance'],
} as const;
