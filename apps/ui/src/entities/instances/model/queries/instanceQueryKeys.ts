export const INSTANCE_QUERY_KEYS = {
  SELF: ['instance'],
  LIST: () => [...INSTANCE_QUERY_KEYS.SELF, 'list'],
  GET: (id: string) => [...INSTANCE_QUERY_KEYS.SELF, 'get', id],
  DIR: (id: string) => [...INSTANCE_QUERY_KEYS.GET(id), 'dir'],
  IMPORTER: {
    SELF: () => [...INSTANCE_QUERY_KEYS.SELF, 'importer'],
    LIST: () => [
      ...INSTANCE_QUERY_KEYS.SELF,
      ...INSTANCE_QUERY_KEYS.IMPORTER.SELF(),
      'list',
    ],
  },
} as const;
