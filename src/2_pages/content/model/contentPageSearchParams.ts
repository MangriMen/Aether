import type { ContentFilters } from '@/entities/instances';

export interface ContentPageSearchParams extends ContentFilters {
  instanceId?: string;
}
