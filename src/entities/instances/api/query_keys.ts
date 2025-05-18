export const QUERY_KEYS = {
  INSTANCE: {
    SELF: ['instance'],
    LIST: () => [...QUERY_KEYS.INSTANCE.SELF, 'list'],
    GET: (id: string) => [...QUERY_KEYS.INSTANCE.SELF, 'get', id],
    DIR: (id: string) => [...QUERY_KEYS.INSTANCE.GET(id), 'dir'],
    PROCESS: {
      SELF: () => [...QUERY_KEYS.INSTANCE.SELF, 'process'],
      BY_INSTANCE: (id: string) => [...QUERY_KEYS.INSTANCE.PROCESS.SELF(), id],
    },
  },
  CONTENT: {
    SELF: ['content'],
    BY_INSTANCE: (id: string) => [...QUERY_KEYS.CONTENT.SELF, 'instance', id],
    PROVIDERS: () => [...QUERY_KEYS.CONTENT.SELF, 'providers'],
    BY_PROVIDER: (provider: string) => [
      ...QUERY_KEYS.CONTENT.PROVIDERS(),
      provider,
    ],
    METADATA_FIELD: (provider: string) => [
      ...QUERY_KEYS.CONTENT.BY_PROVIDER(provider),
      'metadata-field',
    ],
  },
  CONFIG: {
    SELF: ['config'],
    IMPORT: () => [...QUERY_KEYS.CONFIG.SELF, 'import'],
  },
} as const;
