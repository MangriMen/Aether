import type { ContentFilters } from './contentItem';

export interface ContentPageSearchParams extends ContentFilters {
  instanceId: string | undefined;
}
