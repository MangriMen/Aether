import {
  createMemo,
  For,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { type ContentType } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { CombinedPagination, Skeleton } from '@/shared/ui';

import { ContentSearch } from './ContentSearch';
import { ItemsPerPageSelect } from './ItemsPerPageSelect';

export type ContentFiltersProps = ComponentProps<'nav'> & {
  page: number;
  pageSize: number;
  pageCount: number | undefined;
  searchQuery?: string;
  onSearch: (query: string) => void;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  contentType?: ContentType;
  isLoading?: boolean;
};

const DEFAULT_PAGINATION_COUNT_ARRAY = Array.from({ length: 6 });

export const ContentSearchCard: Component<ContentFiltersProps> = (props) => {
  const [local, others] = splitProps(props, [
    'page',
    'pageCount',
    'pageSize',
    'searchQuery',
    'onSearch',
    'onPageChange',
    'onPageSizeChange',
    'contentType',
    'isLoading',
    'class',
  ]);

  const fixedPageCount = createMemo(() =>
    !local.pageCount || local.pageCount < 1 ? 1 : local.pageCount,
  );
  const isPaginationDisabled = createMemo(
    () => !!local.isLoading || !local.pageCount || local.pageCount < 1,
  );

  return (
    <nav
      class={cn('gap-2 flex justify-between', local.class)}
      aria-label='Content pagination'
      {...others}
    >
      <ContentSearch
        value={local.searchQuery ?? ''}
        contentType={local.contentType}
        onSearch={local.onSearch}
      />
      <div class='gap-2 flex justify-between'>
        <ItemsPerPageSelect
          value={local.pageSize}
          onChange={local.onPageSizeChange}
        />

        <Show
          when={local.pageCount !== undefined}
          fallback={
            <div class='gap-1 flex h-full'>
              <For each={DEFAULT_PAGINATION_COUNT_ARRAY}>
                {() => <Skeleton width={40} radius={6} />}
              </For>
            </div>
          }
        >
          <CombinedPagination
            siblingCount={1}
            count={fixedPageCount()}
            page={local.page}
            onPageChange={local.onPageChange}
            disabled={isPaginationDisabled()}
          />
        </Show>
      </div>
    </nav>
  );
};
