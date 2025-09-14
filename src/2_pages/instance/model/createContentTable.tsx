import type {
  ColumnDef,
  ColumnFiltersState,
  PaginationState,
  RowModel,
  RowSelectionState,
  SortingState,
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

import type { InstanceFile } from '@/entities/instances';

import { CONTENT_TABLE_COLUMNS } from './contentTableColumns';

export interface CreateContentTableProps {
  contentActions: Component<{ file: InstanceFile }>;
  data: Accessor<InstanceFile[]>;
  headerActions: Component<{
    allRowsSelected?: boolean;
    refetch?: () => void;
    selectedRows: RowModel<InstanceFile>;
    someRowsSelected?: boolean;
  }>;
  isLoading: Accessor<boolean>;
  refetch: () => void;
}

export const createContentTable = (props: CreateContentTableProps) => {
  const columns = createMemo(() => {
    const actionsColumn: ColumnDef<InstanceFile> = {
      cell: (rowProps) => (
        <div class='flex justify-end'>
          <props.contentActions file={rowProps.cell.row.original} />
        </div>
      ),
      header: (rowProps) => (
        <div class='flex justify-end'>
          <props.headerActions
            allRowsSelected={rowProps.table.getIsAllPageRowsSelected()}
            refetch={props.refetch}
            selectedRows={rowProps.table.getSelectedRowModel()}
            someRowsSelected={rowProps.table.getIsSomePageRowsSelected()}
          />
        </div>
      ),
      id: 'actions',
    };

    return [...CONTENT_TABLE_COLUMNS, actionsColumn];
  });

  const [sorting, setSorting] = createSignal<SortingState>([
    { desc: false, id: 'name' },
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
    get columns() {
      return columns();
    },
    get data() {
      return props.data();
    },
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onColumnFiltersChange: setColumnFilters,
    onPaginationChange: setPagination,
    onRowSelectionChange: setRowSelection,
    onSortingChange: setSorting,
    state: {
      get columnFilters() {
        return columnFilters();
      },
      get pagination() {
        return pagination();
      },
      get rowSelection() {
        return rowSelection();
      },
      get sorting() {
        return sorting();
      },
    },
  });

  return { columns, table };
};
