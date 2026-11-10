export const INSTANCE_QUERY_KEYS = {
  SELF: ['instance'],
  LIST: () => [...INSTANCE_QUERY_KEYS.SELF, 'list'],
  // Under LIST, so what refreshes the list refreshes the broken ones too.
  BROKEN: () => [...INSTANCE_QUERY_KEYS.LIST(), 'broken'],
  GET: (id: string) => [...INSTANCE_QUERY_KEYS.SELF, 'get', id],
  DIR: (id: string) => [...INSTANCE_QUERY_KEYS.GET(id), 'dir'],
  // Content of an instance belongs to the content queries; the key lives here so
  // that instance changes can invalidate it.
  CONTENT: (id: string) => ['content', 'instance', id],
} as const;
