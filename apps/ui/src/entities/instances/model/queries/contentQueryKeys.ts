import type {
  ContentGetParams,
  ContentListVersionParams,
  ContentSearchParams,
  Instance,
  ProviderId,
} from '..';
import type { ContentCompatibilityCheckParams } from '../compatibility';

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
  COMPATIBILITY: (
    ids: Instance['id'][],
    params: ContentCompatibilityCheckParams,
  ) => [
    ...CONTENT_QUERY_KEYS.BY_PROVIDER(params.providerId),
    'compatibility',
    ids,
    params.contentItem.id,
  ],
  GET: (params: ContentGetParams) => [
    ...CONTENT_QUERY_KEYS.BY_PROVIDER(params.providerId),
    'get',
    params.contentId,
  ],
  GET_EMPTY: () => [...CONTENT_QUERY_KEYS.SELF, 'get', 'empty'],
  LIST_CONTENT_VERSION: (params: ContentListVersionParams) => [
    ...CONTENT_QUERY_KEYS.BY_PROVIDER(params.providerId),
    'list_version',
    params.contentId,
  ],
  LIST_CONTENT_VERSION_EMPTY: () => [
    ...CONTENT_QUERY_KEYS.SELF,
    'list_version',
    'empty',
  ],
} as const;
