import type { Component } from 'solid-js';

import { createEffect } from 'solid-js';

import { type ContentFile } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { CombinedPagination, DataTable } from '@/shared/ui';

import { createContentTable } from '../model';
import { ContentActions } from './ContentActions';
import { HeaderActions } from './HeaderActions';

export type ContentTableProps = {
  data: ContentFile[];
  searchQuery?: string;
  isLoading?: boolean;
  refetch?: () => void;
  instanceId: string;
  instancePath?: string;
};

export const ContentTable: Component<ContentTableProps> = (props) => {
  const { table, columns } = createContentTable({
    headerActions: (localProps) => (
      <HeaderActions instanceId={props.instanceId} {...localProps} />
    ),
    contentActions: (localProps) => (
      <ContentActions
        instanceId={props.instanceId}
        instancePath={props.instancePath}
        content={localProps.file}
      />
    ),
    data: () => props.data,
    refetch: () => props.refetch?.(),
    isLoading: () => !!props.isLoading,
  });

  const handleSearch = (query: string) =>
    table.getColumn('name')?.setFilterValue(query);

  createEffect(() => {
    if (props.searchQuery === undefined) {
      return;
    }
    handleSearch(props.searchQuery);
  });

  return (
    <>
      <DataTable columns={columns()} table={table} />
      <CombinedPagination
        class={cn('self-end', { hidden: table.getPageCount() < 2 })}
        siblingCount={1}
        count={table.getPageCount()}
        page={table.getState().pagination.pageIndex + 1}
        onPageChange={(page) => table.setPageIndex(page - 1)}
      />
    </>
  );
};
