import type { ContentSearchParams, ProviderId } from '../../model';

export const CONTENT_QUERY_KEYS = {
  SELF: ['content'],
  BY_INSTANCE: (id: string) => [...CONTENT_QUERY_KEYS.SELF, 'instance', id],
  PROVIDERS: () => [...CONTENT_QUERY_KEYS.SELF, 'providers'],
  BY_PROVIDER: (provider: ProviderId) => [
    ...CONTENT_QUERY_KEYS.PROVIDERS(),
    provider.pluginId,
    provider.capabilityId,
  ],
  SEARCH: (params: ContentSearchParams) => [
    ...CONTENT_QUERY_KEYS.BY_PROVIDER(params.providerId),
    'search',
    params?.page,
    params?.pageSize,
    params?.query,
    params?.contentType,
    params?.gameVersions?.toString(),
    params?.loader,
  ],
  SEARCH_EMPTY: () => [...CONTENT_QUERY_KEYS.SELF, 'search', 'empty'],
} as const;
