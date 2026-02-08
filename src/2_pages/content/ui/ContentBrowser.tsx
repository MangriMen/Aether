import { debounce } from '@solid-primitives/scheduled';
import {
  createEffect,
  createMemo,
  createSignal,
  Match,
  mergeProps,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type {
  ContentFilters,
  ContentRequest,
  Instance,
} from '@/entities/instances';
import type { ContentProviderCapabilityMetadata } from '@/entities/instances/model/capabilities';
import type { CapabilityEntry, Option } from '@/shared/model';

import {
  CONTENT_TYPES,
  ContentType,
  useSearchContent,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedSelect } from '@/shared/ui';

import type { ContentFiltersLock } from '../model/contentFiltersLock';

import { ContentContextProvider } from './ContentContextProvider';
import { ContentList } from './ContentList';
import { ContentListSkeleton } from './ContentListSkeleton';
import { ContentSearchCard } from './ContentSearchCard';
import { ContentTypeTabs } from './ContentTypeTabs';

export type ContentBrowserProps = ComponentProps<'div'> & {
  providers: Option<CapabilityEntry<ContentProviderCapabilityMetadata>>[];
  types?: ContentType[];
  instance?: Instance;
  filters?: ContentFilters;
  filtersLock?: ContentFiltersLock;
};

export const ContentBrowser: Component<ContentBrowserProps> = (props) => {
  const [_local, others] = splitProps(props, [
    'providers',
    'instance',
    'types',
    'filters',
    'filtersLock',
    'class',
  ]);

  const local = mergeProps(
    {
      types: CONTENT_TYPES,
    },
    _local,
  );

  const [{ t }] = useTranslation();

  const [contentType, setContentType] = createSignal<ContentType>(
    ContentType.Mod,
  );
  createEffect(() => {
    setContentType(local.types[0]);
  });

  const [searchQuery, setSearchQuery] = createSignal<string | undefined>();
  const [provider, setProvider] = createSignal<Option<
    CapabilityEntry<ContentProviderCapabilityMetadata>
  > | null>(null);
  const [page, setPage] = createSignal(1);
  const [pageSize, setPageSize] = createSignal(20);

  createEffect(() => {
    if (local.providers.length) {
      setProvider(local.providers[0]);
    }
  });

  const handleSearch = debounce((query: string) => {
    if (query.trim()) {
      setPage(1);
    }
    setSearchQuery(query);
  }, 300);
  const handlePageChange = (page: number) => setPage(page);
  const handlePageSizeChange = (pageSize: number) => setPageSize(pageSize);
  const handleSetProvider = (
    provider: Option<CapabilityEntry<ContentProviderCapabilityMetadata>> | null,
  ) => {
    if (provider) {
      setProvider(provider);
    }
  };

  const contentRequestPayload = createMemo((): ContentRequest | undefined => {
    const currentProvider = provider();
    if (!currentProvider) {
      return;
    }

    return {
      contentType: contentType(),
      provider: currentProvider.value.capability.id,
      page: page(),
      pageSize: pageSize(),
      query: searchQuery(),
      gameVersions: local.filters?.gameVersions,
      loader: local.filters?.loaders?.[0],
    };
  });

  const content = useSearchContent(() => contentRequestPayload());

  return (
    <ContentContextProvider
      providerId={provider()?.value.capability.id}
      providerDataContentIdField={
        provider()?.value.capability.providerDataContentIdField
      }
      filters={local.filters}
      instanceId={local.instance?.id}
    >
      <div
        class={cn('flex flex-col grow gap-2 overflow-hidden p-1', local.class)}
        {...others}
      >
        <div class='flex justify-between gap-2'>
          <ContentTypeTabs onChange={setContentType} items={local.types} />
          <div class='flex items-center gap-2'>
            <span class='text-muted-foreground'>{t('content.provider')}:</span>
            <CombinedSelect
              options={local.providers}
              value={provider()}
              onChange={handleSetProvider}
              optionValue='value'
              optionTextValue='name'
              title={t('content.provider')}
            />
          </div>
        </div>
        <ContentSearchCard
          pageSize={pageSize()}
          pageCount={content.data?.pageCount}
          currentPage={page()}
          onSearch={handleSearch}
          onPageChange={handlePageChange}
          onPageSizeChange={handlePageSizeChange}
          contentType={contentType()}
          loading={content.isFetching}
        />
        <Switch>
          <Match when={content.isFetching}>
            <ContentListSkeleton />
          </Match>
          <Match when={content.isError}>
            <span class='flex grow items-center justify-center text-xl font-medium text-muted-foreground'>
              {t('content.providerErrorOrNotFound')}
            </span>
          </Match>
          <Match when={content.data?.items ?? []}>
            {(items) => (
              <ContentList items={items()} instanceId={local.instance?.id} />
            )}
          </Match>
        </Switch>
      </div>
    </ContentContextProvider>
  );
};
