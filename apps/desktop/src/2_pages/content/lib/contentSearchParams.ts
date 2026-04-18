import type { SearchParams } from '@solidjs/router';

import { isContentType } from '@/entities/instances';
import {
  parseSearchParamToNumber,
  parseSearchParamToString,
  parseSearchParamToStringArray,
} from '@/shared/lib';

import type { ContentPageSearchParams } from '../model/contentPageSearchParams';

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
    page: parseSearchParamToNumber(page),
    pageSize: parseSearchParamToNumber(pageSize),
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
