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
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { ContentFilters } from './ContentFilters';
import { ContentList } from './ContentList';
import { CONTENT_TYPE_TO_TITLE } from '../model';

export type ContentBrowserProps = ComponentProps<'div'> & {
  availableContent?: ContentType[];
};

const PROVIDERS = ['modrinth', 'curseforge'];

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

  const handleOnSearch = (query: string) => setSearchQuery(query);
  const handlePageChange = (page: number) => setCurrentPage(page);
  const handlePageSizeChange = (pageSize: number) =>
    setCurrentPageSize(pageSize);

  const contentRequestPayload = createMemo<ContentRequest>(() => ({
    contentType: currentContentType(),
    provider: currentProvider(),
    page: currentPage(),
    pageSize: currentPageSize(),
    query: searchQuery(),
  }));

  const [contentRequest] = createResource(contentRequestPayload, (payload) =>
    getContentByProvider(payload).catch((e) => {
      console.error(e);
      return undefined;
    }),
  );

  return (
    <div
      class={cn('flex flex-col gap-2 overflow-hidden p-1', local.class)}
      {...others}
    >
      <div class='flex gap-2'>
        <CombinedSelect
          options={PROVIDERS}
          value={currentProvider()}
          onChange={setCurrentProvider}
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
      <ContentList items={contentRequest()?.items ?? []} />
    </div>
  );
};
