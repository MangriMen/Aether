import type { ContentFilters, ContentType } from '../../../entities/instances';
import type { DEFAULT_FORM_VALUES } from '../model';

export const resolveContentFilters = (
  current: ContentFilters | undefined,
  updates: Partial<ContentFilters>,
  defaults: typeof DEFAULT_FORM_VALUES,
  defaultContentType: ContentType,
): ContentFilters => {
  const isPaginationOnly =
    updates.page !== undefined && Object.keys(updates).length === 1;

  // Reset page to 1 if any filter other than page itself is changed
  const next: ContentFilters = {
    ...current,
    ...updates,
    page: isPaginationOnly ? updates.page : defaults.page,
  };

  return {
    ...next,
    page: next.page === defaults.page ? undefined : next.page,
    pageSize: next.pageSize === defaults.pageSize ? undefined : next.pageSize,
    query:
      next.query === '' || next.query === defaults.query
        ? undefined
        : next.query,
    contentType:
      next.contentType === defaultContentType ? undefined : next.contentType,
  };
};
