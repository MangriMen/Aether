import {
  createEffect,
  createMemo,
  createSignal,
  mergeProps,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentSearchParams } from '@/entities/instances';
import type { ContentType } from '@/entities/instances';
import type { Option } from '@/shared/model';

import {
  CONTENT_TYPES,
  isModLoader,
  useSearchContent,
} from '@/entities/instances';
import { cn, debounce } from '@/shared/lib';

import type { ContentProviderEntry } from '../model';

import { useContentBrowserFilters } from '../lib/useContentBrowserFilters';
import { SEARCH_QUERY_DEBOUNCE_DELAY } from '../model/constants';
import { ContentList } from './ContentList';
import { ContentProviderSelect } from './ContentProviderSelect';
import { ContentSearchCard } from './ContentSearchCard';
import { ContentTypeTabs } from './ContentTypeTabs';

export type ContentBrowserProps = ComponentProps<'div'> & {
  providers: Option<ContentProviderEntry>[];
  isProvidersLoading?: boolean;
  isProvidersError?: boolean;
  types?: readonly ContentType[];
};

export const ContentBrowser: Component<ContentBrowserProps> = (props) => {
  const merged = mergeProps({ types: CONTENT_TYPES }, props);

  const [local, others] = splitProps(merged, [
    'providers',
    'isProvidersLoading',
    'types',
    'class',
  ]);

  const { state, actions } = useContentBrowserFilters(
    () => local.providers,
    () => local.types,
  );

  const [localQuery, setLocalQuery] = createSignal('');
  createEffect(() => setLocalQuery(state.query()));

  const debounceSetQuery = debounce(
    (query: string) => actions.setQuery(query),
    SEARCH_QUERY_DEBOUNCE_DELAY,
  );

  const handleSearch = (query: string) => {
    setLocalQuery(query);
    debounceSetQuery(query);
  };

  const contentSearchParams = createMemo(
    (): ContentSearchParams | undefined => {
      const currentProvider = state.provider()?.value;

      if (!currentProvider) {
        return;
      }

      const loader = state.loader();

      return {
        contentType: state.contentType(),
        providerId: {
          pluginId: currentProvider.pluginId,
          capabilityId: currentProvider.capability.id,
        },
        page: state.page(),
        pageSize: state.pageSize(),
        query: state.query(),
        gameVersions: state.gameVersions(),
        loader: isModLoader(loader) ? loader : undefined,
      };
    },
  );

  const content = useSearchContent(() => contentSearchParams());

  return (
    <div
      class={cn('flex flex-col grow gap-3 overflow-hidden p-0.5', local.class)}
      {...others}
    >
      <div class='flex flex-col gap-2'>
        <div class='flex justify-between gap-2'>
          <ContentTypeTabs
            items={local.types}
            value={state.contentType()}
            onChange={actions.setContentType}
            disabled={local.isProvidersLoading}
            isLoading={local.types.length === 0 || local.isProvidersLoading}
          />
          <ContentProviderSelect
            options={local.providers}
            value={state.provider()}
            onChange={actions.setProvider}
            isLoading={local.isProvidersLoading}
          />
        </div>

        <ContentSearchCard
          page={state.page()}
          pageSize={state.pageSize()}
          pageCount={content.data?.pageCount}
          contentType={state.contentType()}
          searchQuery={localQuery()}
          isLoading={content.isFetching || local.isProvidersLoading}
          onSearch={handleSearch}
          onPageChange={actions.setPage}
          onPageSizeChange={actions.setPageSize}
        />
      </div>

      <ContentList
        isLoading={content.isFetching || local.isProvidersLoading}
        isError={content.isError}
        items={content.data?.items}
      />
    </div>
  );
};
