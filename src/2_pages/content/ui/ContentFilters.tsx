import {
  createMemo,
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

export type ContentFiltersProps = ComponentProps<'div'> & {
  pageCount: number | undefined;
  pageSize: number;
  currentPage: number;
  onSearch: (query: string) => void;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  contentType?: ContentType;
  loading?: boolean;
};

export const ContentFilters: Component<ContentFiltersProps> = (props) => {
  const [local, others] = splitProps(props, [
    'pageCount',
    'pageSize',
    'currentPage',
    'onSearch',
    'onPageChange',
    'onPageSizeChange',
    'contentType',
    'loading',
    'class',
  ]);

  const fixedPageCount = createMemo(() =>
    !local.pageCount || local.pageCount < 1 ? 1 : local.pageCount,
  );
  const isPaginationDisabled = createMemo(
    () => !!local.loading || !local.pageCount || local.pageCount < 1,
  );

  return (
    <div class={cn('flex gap-2 justify-between', local.class)} {...others}>
      <ContentSearch
        contentType={local.contentType}
        onSearch={local.onSearch}
      />
      <div class='flex justify-between gap-2'>
        <ItemsPerPageSelect
          value={local.pageSize}
          onChange={local.onPageChange}
        />

        <Show
          when={local.pageCount !== undefined}
          fallback={<Skeleton width={200} radius={6} class='bg-secondary' />}
        >
          <CombinedPagination
            siblingCount={1}
            count={fixedPageCount()}
            page={local.currentPage}
            onPageChange={local.onPageChange}
            disabled={isPaginationDisabled()}
          />
        </Show>
      </div>
    </div>
  );
};
