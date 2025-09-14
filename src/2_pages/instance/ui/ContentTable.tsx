import type { Component } from 'solid-js';

import { createEffect } from 'solid-js';

import { type InstanceFile } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { CombinedPagination, DataTable } from '@/shared/ui';

import { createContentTable } from '../model';
import { ContentActions } from './ContentActions';
import { HeaderActions } from './HeaderActions';

export type ContentTableProps = {
  data: InstanceFile[];
  instanceId: string;
  instancePath?: string;
  isLoading?: boolean;
  refetch?: () => void;
  searchQuery?: string;
};

export const ContentTable: Component<ContentTableProps> = (props) => {
  const { columns, table } = createContentTable({
    contentActions: (localProps) => (
      <ContentActions
        content={localProps.file}
        instanceId={props.instanceId}
        instancePath={props.instancePath}
      />
    ),
    data: () => props.data,
    headerActions: (localProps) => (
      <HeaderActions instanceId={props.instanceId} {...localProps} />
    ),
    isLoading: () => !!props.isLoading,
    refetch: () => props.refetch?.(),
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
        count={table.getPageCount()}
        onPageChange={(page) => table.setPageIndex(page - 1)}
        page={table.getState().pagination.pageIndex + 1}
        siblingCount={1}
      />
    </>
  );
};
