import { isContentType } from '@/entities/instances';
import {
  parseSearchParamToNumber,
  parseSearchParamToString,
  parseSearchParamToStringArray,
} from '@/shared/lib/parseSearchParamsField';

import type { ContentPageSearchParams } from '../model/contentPageSearchParams';

export const parseSearchParams = <
  T extends Partial<Record<string, string | string[] | undefined>>,
>(
  searchParams: T,
): ContentPageSearchParams => {
  const contentTypeString = parseSearchParamToString(
    searchParams['contentType'],
  );

  return {
    instanceId: parseSearchParamToString(searchParams['instance']),
    page: parseSearchParamToNumber(searchParams['page']),
    pageSize: parseSearchParamToNumber(searchParams['pageSize']),
    query: parseSearchParamToString(searchParams['query']),
    provider: parseSearchParamToString(searchParams['provider']),
    contentType: isContentType(contentTypeString)
      ? contentTypeString
      : undefined,
    gameVersions: parseSearchParamToStringArray(searchParams['gameVersions']),
    loaders: parseSearchParamToStringArray(searchParams['loaders']),
  };
};
