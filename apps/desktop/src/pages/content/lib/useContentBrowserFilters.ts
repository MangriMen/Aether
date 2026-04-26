import type { Accessor } from 'solid-js';

import { createEffect, createMemo } from 'solid-js';

import type { ContentFilters, ContentType } from '@/entities/instances';
import type { Option } from '@/shared/model';

import {
  DEFAULT_FORM_VALUES,
  useContentContext,
  type ContentProviderEntry,
} from '../model';
import { isFiltersEqual } from './isFilterEqual';
import { resolveContentFilters } from './resolveContentFilters';

// useContentBrowserFilters.ts
export const useContentBrowserFilters = (
  providers: Accessor<Option<ContentProviderEntry>[]>,
  types: Accessor<readonly ContentType[]>,
) => {
  const [context, { setFilters: onFiltersChange }] = useContentContext();

  const defaultContentType = () =>
    types()[0] ?? DEFAULT_FORM_VALUES.contentType;

  const setFilters = (updates: Partial<ContentFilters>) => {
    if (providers().length === 0) {
      return;
    }

    const current = context.filters;

    const final = resolveContentFilters(
      current,
      updates,
      DEFAULT_FORM_VALUES,
      defaultContentType(),
    );

    if (!final.providerId && current?.providerId) {
      final.providerId = current.providerId;
    }

    if (!isFiltersEqual(context.filters, final)) {
      onFiltersChange?.(final);
    }
  };

  const currentProvider = createMemo(
    (): Option<ContentProviderEntry> | undefined => {
      const currentProviders = providers();

      const providerId = context.filters?.providerId;

      if (!providerId) {
        return;
      }

      return currentProviders.find(
        (p) =>
          p.value.pluginId === providerId.pluginId &&
          p.value.capability.id === providerId.capabilityId,
      );
    },
  );

  const setProvider = (provider: Option<ContentProviderEntry> | null) => {
    if (provider)
      setFilters({
        providerId: {
          pluginId: provider.value.pluginId,
          capabilityId: provider.value.capability.id,
        },
      });
  };

  createEffect(() => {
    const currentProviders = providers();
    const currentProviderId = context.filters?.providerId;

    if (currentProviders.length > 0 && !currentProviderId) {
      const defaultProvider = currentProviders[0];
      setProvider(defaultProvider);
    }
  });

  return {
    state: {
      contentType: () => context.filters?.contentType ?? types()[0],
      provider: currentProvider,
      page: () => context.filters?.page ?? DEFAULT_FORM_VALUES.page,
      pageSize: () => context.filters?.pageSize ?? DEFAULT_FORM_VALUES.pageSize,
      query: () => context.filters?.query ?? DEFAULT_FORM_VALUES.query,
      gameVersions: () => context.filters?.gameVersions,
      loader: () => context.filters?.loaders?.[0],
    },
    actions: {
      setContentType: (contentType: ContentType) => setFilters({ contentType }),
      setPage: (page: number) => setFilters({ page }),
      setPageSize: (pageSize: number) => setFilters({ pageSize }),
      setQuery: (query: string) => setFilters({ query }),
      setProvider,
    },
  };
};
