import type { ContentPageSearchParams } from '../model/contentPageSearchParams';

export const parseSearchParams = <
  T extends Partial<Record<string, string | string[] | undefined>>,
>(
  searchParams: T,
): ContentPageSearchParams => {
  const instanceIdRaw = searchParams['instance'];
  const instanceId =
    typeof instanceIdRaw === 'string'
      ? decodeURIComponent(instanceIdRaw)
      : undefined;

  return {
    instanceId,
  };
};
