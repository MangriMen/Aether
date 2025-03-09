import {
  type ContentItem,
  CONTENT_TYPES,
  type ContentType,
} from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslate } from '@/shared/model';
import { Tabs, TabsList, TabsTrigger } from '@/shared/ui';
import {
  createMemo,
  createSignal,
  For,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { ContentFilters } from './ContentFilters';
import { ContentList } from './ContentList';

export type ContentBrowserProps = ComponentProps<'div'> & {
  availableContent?: ContentType[];
};

const CONTENT_TYPE_TO_TITLE: Record<
  ContentType,
  'mods' | 'dataPacks' | 'resourcePacks' | 'shaders'
> = {
  mod: 'mods',
  datapack: 'dataPacks',
  resourcepack: 'resourcePacks',
  shaderpack: 'shaders',
} as const;

export const ContentBrowser: Component<ContentBrowserProps> = (props) => {
  const [local, others] = splitProps(props, ['availableContent', 'class']);

  const [{ t }] = useTranslate();

  const [_currentContentType, setCurrentContentType] =
    createSignal<ContentType>(CONTENT_TYPES[0]);

  const availableContentTabs = createMemo(() => {
    return local.availableContent ?? CONTENT_TYPES;
  });

  const [currentPage, setCurrentPage] = createSignal(1);

  const [currentPageSize, setCurrentPageSize] = createSignal(20);

  const handleOnSearch = (_query: string) => {};

  const handlePageChange = (_page: number) => {
    setCurrentPage(_page);
  };

  const handlePageSizeChange = (_pageSize: number) => {
    setCurrentPageSize(_pageSize);
  };

  const items = createMemo(() => {
    const length = currentPageSize();
    return Array.from({ length }).map((_, i) => ({
      id: `mock-content-item-${i}`,
      name: `Mock content item ${i}`,
      description: `This is a mock content item ${i}`,
      type: _currentContentType(),
      url: `https://example.com/content-item-${i}`,
      author: 'Mock author',
    })) as ContentItem[];
  });

  return (
    <div
      class={cn('flex flex-col gap-2 overflow-hidden px-1', local.class)}
      {...others}
    >
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
      <ContentFilters
        pageSize={currentPageSize()}
        pageCount={552}
        currentPage={currentPage()}
        onSearch={handleOnSearch}
        onPageChange={handlePageChange}
        onPageSizeChange={handlePageSizeChange}
      />
      <ContentList items={items()} />
    </div>
  );
};
