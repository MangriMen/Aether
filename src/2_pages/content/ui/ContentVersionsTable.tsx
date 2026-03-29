import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { ContentType, ContentVersion } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { CombinedPagination, DataTable } from '@/shared/ui';

import { createContentVersionsTable } from '../lib/createContentVersionsTable';

export type ContentVersionsTableProps = ComponentProps<'div'> & {
  data: ContentVersion[];
  isLoading?: boolean;
  slug?: string;
  contentType?: ContentType;
};

export const ContentVersionsTable: Component<ContentVersionsTableProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'data',
    'isLoading',
    'slug',
    'contentType',
    'class',
  ]);

  const { table, columns } = createContentVersionsTable({
    data: () => local.data,
    slug: () => local.slug,
    contentType: () => local.contentType,
  });

  return (
    <div
      class={cn('flex flex-col overflow-auto gap-4 p-1', local.class)}
      {...others}
    >
      <DataTable columns={columns()} table={table} />
      <CombinedPagination
        class={cn('self-end', { hidden: table.getPageCount() < 2 })}
        siblingCount={1}
        count={table.getPageCount()}
        page={table.getState().pagination.pageIndex + 1}
        onPageChange={(page) => table.setPageIndex(page - 1)}
      />
    </div>
  );
};
