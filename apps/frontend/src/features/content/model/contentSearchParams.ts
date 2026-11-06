import type { SearchParams } from '@solidjs/router';

import {
  parseSearchParamToNumber,
  parseSearchParamToString,
  parseSearchParamToStringArray,
  searchParamsToQueryString,
} from '@/shared/search-params';

import type { ContentPageSearchParams } from './contentPageSearchParams';

import { CONTENT_ROUTES } from './contentRoutes';
import { isContentType } from './contentType';

/** The page sizes the page offers; the backend refuses any other. */
export const CONTENT_PAGE_SIZES = [5, 10, 20, 30, 40, 50];

const normalizePage = (value: number | undefined): number | undefined =>
  value !== undefined && Number.isInteger(value) && value >= 1
    ? value
    : undefined;

const normalizePageSize = (value: number | undefined): number | undefined =>
  value !== undefined && CONTENT_PAGE_SIZES.includes(value) ? value : undefined;

export const decodeContentSearchParams = (
  searchParams: Partial<SearchParams>,
): ContentPageSearchParams => {
  const {
    contentType: contentType_,
    instance,
    page,
    pageSize,
    query,
    plugin,
    capability,
    gameVersions,
    loaders,
  } = searchParams;

  const contentTypeString = parseSearchParamToString(contentType_);

  const contentType = isContentType(contentTypeString)
    ? contentTypeString
    : undefined;

  const pluginId = parseSearchParamToString(plugin);
  const capabilityId = parseSearchParamToString(capability);
  const provider =
    pluginId && capabilityId ? { pluginId, capabilityId } : undefined;

  return {
    instanceId: parseSearchParamToString(instance),
    page: normalizePage(parseSearchParamToNumber(page)),
    pageSize: normalizePageSize(parseSearchParamToNumber(pageSize)),
    query: parseSearchParamToString(query),
    providerId: provider,
    contentType,
    gameVersions: parseSearchParamToStringArray(gameVersions),
    loaders: parseSearchParamToStringArray(loaders),
  };
};

export const encodeContentSearchParams = (
  filters: Partial<ContentPageSearchParams>,
): SearchParams => {
  return {
    instance: filters.instanceId,
    page: filters.page?.toString(),
    pageSize: filters.pageSize?.toString(),
    query: filters.query || undefined,
    plugin: filters.providerId?.pluginId,
    capability: filters.providerId?.capabilityId,
    contentType: filters.contentType,
    gameVersions: filters.gameVersions?.length
      ? filters.gameVersions
      : undefined,
    loaders: filters.loaders?.length ? filters.loaders : undefined,
  };
};

export const createContentPageHref = (
  contentId: string,
  params: {
    instanceId: string | undefined;
    providerId: { pluginId: string; capabilityId: string } | undefined;
  },
): string => {
  const encodedFilters = encodeContentSearchParams({
    instanceId: params.instanceId,
    providerId: params.providerId,
  });
  const queryString = searchParamsToQueryString(encodedFilters);

  const baseUrl = CONTENT_ROUTES.CONTENT_ITEM(contentId);

  if (!queryString) {
    return baseUrl;
  }

  return `${baseUrl}?${queryString}`;
};
