import { CONTENT_TYPES, getContentByProvider } from '@/entities/instances';

import type { ContentRequest, ContentType } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslate } from '@/shared/model';
import { CombinedSelect, Tabs, TabsList, TabsTrigger } from '@/shared/ui';
import {
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { ContentFilters } from './ContentFilters';
import { ContentList } from './ContentList';
import { CONTENT_TYPE_TO_TITLE } from '../model';
import { debounce } from '@solid-primitives/scheduled';
import { ContentListSkeleton } from './ContentListSkeleton';

export type ContentBrowserProps = ComponentProps<'div'> & {
  availableContent?: ContentType[];
};

const PROVIDERS = ['curseforge', 'modrinth'];

export const ContentBrowser: Component<ContentBrowserProps> = (props) => {
  const [local, others] = splitProps(props, ['availableContent', 'class']);

  const [{ t }] = useTranslate();

  const [currentContentType, setCurrentContentType] = createSignal<ContentType>(
    CONTENT_TYPES[0],
  );

  const availableContentTabs = createMemo(
    () => local.availableContent ?? CONTENT_TYPES,
  );

  const [searchQuery, setSearchQuery] = createSignal<string | undefined>();
  const [currentProvider, setCurrentProvider] = createSignal(PROVIDERS[0]);
  const [currentPage, setCurrentPage] = createSignal(1);
  const [currentPageSize, setCurrentPageSize] = createSignal(20);

  const handleOnSearch = debounce(
    (query: string) => setSearchQuery(query),
    300,
  );
  const handlePageChange = (page: number) => setCurrentPage(page);
  const handlePageSizeChange = (pageSize: number) =>
    setCurrentPageSize(pageSize);

  const handleSetContentProvider = (provider: string | null) => {
    if (provider) {
      setCurrentProvider(provider);
    }
  };

  const contentRequestPayload = createMemo<ContentRequest>(() => ({
    contentType: currentContentType(),
    provider: currentProvider(),
    page: currentPage(),
    pageSize: currentPageSize(),
    query: searchQuery(),
  }));

  const [contentRequestError, setContentRequestError] = createSignal<
    unknown | null
  >(null);
  const setContentRequestErrorDebounced = debounce(setContentRequestError, 100);

  const getContent = async (payload: ContentRequest) => {
    setContentRequestErrorDebounced(null);
    try {
      return await getContentByProvider(payload);
    } catch (e) {
      setContentRequestErrorDebounced(e);
    }
  };

  const [contentRequest] = createResource(contentRequestPayload, getContent);

  return (
    <div
      class={cn('flex flex-col grow gap-2 overflow-hidden p-1', local.class)}
      {...others}
    >
      <div class='flex gap-2'>
        <CombinedSelect
          options={PROVIDERS}
          value={currentProvider()}
          onChange={handleSetContentProvider}
        />
        <Tabs onChange={setCurrentContentType}>
          <TabsList>
            <For each={availableContentTabs()}>
              {(contentType) => (
                <TabsTrigger value={contentType}>
                  {t(`content.${CONTENT_TYPE_TO_TITLE[contentType]}`)}
                </TabsTrigger>
              )}
            </For>
          </TabsList>
        </Tabs>
      </div>
      <ContentFilters
        pageSize={currentPageSize()}
        pageCount={contentRequest()?.pageCount ?? 0}
        currentPage={currentPage()}
        onSearch={handleOnSearch}
        onPageChange={handlePageChange}
        onPageSizeChange={handlePageSizeChange}
        contentType={currentContentType()}
      />
      <Show
        when={!contentRequestError()}
        fallback={
          <span class='flex grow items-center justify-center text-xl font-medium text-muted-foreground'>
            {t('content.providerErrorOrNotFound')}
          </span>
        }
      >
        <Show when={contentRequest()} fallback={<ContentListSkeleton />}>
          {(contentRequest) => <ContentList items={contentRequest().items} />}
        </Show>
      </Show>
    </div>
  );
};
