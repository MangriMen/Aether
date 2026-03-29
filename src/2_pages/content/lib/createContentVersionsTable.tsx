import type {
  ColumnDef,
  ColumnFiltersState,
  PaginationState,
  RowSelectionState,
  SortingState,
} from '@tanstack/solid-table';
import type { Accessor } from 'solid-js';

import {
  createSolidTable,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
} from '@tanstack/solid-table';
import { createMemo, createSignal } from 'solid-js';

import type { ContentType, ContentVersion } from '@/entities/instances';

import { CONTENT_VERSIONS_TABLE_COLUMNS } from '../model/contentVersionsColumns';
import { ContentVersionActions } from '../ui/ContentVersionActions';

export interface ContentVersionsTableProps {
  data: Accessor<ContentVersion[]>;
  slug: Accessor<string | undefined>;
  contentType: Accessor<ContentType | undefined>;
}

export const createContentVersionsTable = (
  props: ContentVersionsTableProps,
) => {
  const [sorting, setSorting] = createSignal<SortingState>([
    { desc: true, id: 'datePublished' },
  ]);
  const [columnFilters, setColumnFilters] = createSignal<ColumnFiltersState>(
    [],
  );
  const [rowSelection, setRowSelection] = createSignal<RowSelectionState>({});
  const [pagination, setPagination] = createSignal<PaginationState>({
    pageIndex: 0,
    pageSize: 20,
  });

  const columns = createMemo((): ColumnDef<ContentVersion>[] => {
    return [
      ...CONTENT_VERSIONS_TABLE_COLUMNS,
      {
        id: 'actions',
        cell: (cellProps) => (
          <ContentVersionActions
            version={cellProps.row.original}
            slug={props.slug()}
            contentType={props.contentType()}
          />
        ),
      },
    ];
  });

  const table = createSolidTable({
    get data() {
      return props.data();
    },
    get columns() {
      return columns();
    },
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onRowSelectionChange: setRowSelection,
    onPaginationChange: setPagination,
    state: {
      get pagination() {
        return pagination();
      },
      get sorting() {
        return sorting();
      },
      get columnFilters() {
        return columnFilters();
      },
      get rowSelection() {
        return rowSelection();
      },
    },
  });

  return { table, columns };
};
