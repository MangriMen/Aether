import type {
  ColumnFiltersState,
  SortingState,
  RowSelectionState,
  PaginationState,
  RowModel,
} from '@tanstack/solid-table';
import type { Accessor, Component } from 'solid-js';

import {
  createSolidTable,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
} from '@tanstack/solid-table';
import { createMemo, createSignal } from 'solid-js';

import type { ContentFile } from '../../../entities/instances';

import {
  CONTENT_TABLE_COLUMNS,
  contentTableColumnHelper,
} from './contentTableColumns';

export interface CreateContentTableProps {
  headerActions: Component<{
    allRowsSelected?: boolean;
    someRowsSelected?: boolean;
    refetch?: () => void;
    selectedRows: RowModel<ContentFile>;
  }>;
  contentActions: Component<{ file: ContentFile }>;
  data: Accessor<ContentFile[]>;
  refetch: () => void;
  isLoading: Accessor<boolean>;
}

export const createContentTable = (props: CreateContentTableProps) => {
  const columns = createMemo(() => {
    const actionsColumn = contentTableColumnHelper.display({
      id: 'actions',
      size: 116,
      maxSize: 116,
      header: (headerProps) => (
        <div class='flex justify-end'>
          <props.headerActions
            allRowsSelected={headerProps.table.getIsAllPageRowsSelected()}
            someRowsSelected={headerProps.table.getIsSomePageRowsSelected()}
            selectedRows={headerProps.table.getSelectedRowModel()}
            refetch={props.refetch}
          />
        </div>
      ),
      cell: (cellProps) => (
        <div class='flex justify-end'>
          {/* Используем сокращенный путь props.row.original */}
          <props.contentActions file={cellProps.row.original} />
        </div>
      ),
    });

    return [...CONTENT_TABLE_COLUMNS, actionsColumn];
  });

  const [sorting, setSorting] = createSignal<SortingState>([
    { desc: true, id: 'name' },
  ]);
  const [columnFilters, setColumnFilters] = createSignal<ColumnFiltersState>(
    [],
  );
  const [rowSelection, setRowSelection] = createSignal<RowSelectionState>({});
  const [pagination, setPagination] = createSignal<PaginationState>({
    pageIndex: 0,
    pageSize: 20,
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
