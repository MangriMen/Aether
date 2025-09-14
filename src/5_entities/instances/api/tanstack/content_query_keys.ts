export const CONTENT_QUERY_KEYS = {
  BY_INSTANCE: (id: string) => [...CONTENT_QUERY_KEYS.SELF, 'instance', id],
  BY_PROVIDER: (provider: string) => [
    ...CONTENT_QUERY_KEYS.PROVIDERS(),
    provider,
  ],
  METADATA_FIELD: (provider: string) => [
    ...CONTENT_QUERY_KEYS.BY_PROVIDER(provider),
    'metadata-field',
  ],
  PROVIDERS: () => [...CONTENT_QUERY_KEYS.SELF, 'providers'],
  SELF: ['content'],
} as const;
