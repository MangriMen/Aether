import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { ContentType, ContentVersion } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { CombinedPagination, DataTable } from '@/shared/ui';

import { createContentVersionsTable } from '../lib/createContentVersionsTable';

export type ContentVersionsTableProps = ComponentProps<'div'> & {
  data: ContentVersion[];
  isLoading?: boolean;
  contentType?: ContentType;
};

export const ContentVersionsTable: Component<ContentVersionsTableProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'data',
    'isLoading',
    'contentType',
    'class',
  ]);

  const { table, columns } = createContentVersionsTable({
    data: () => local.data,
    contentType: () => local.contentType,
  });

  return (
    <div
      class={cn('flex flex-col gap-4 p-1 overflow-hidden', local.class)}
      {...others}
    >
      <DataTable
        columns={columns()}
        table={table}
        isLoading={local.isLoading}
      />
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
