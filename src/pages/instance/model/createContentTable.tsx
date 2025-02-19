import type { Accessor, Component } from 'solid-js';
import { createMemo, createSignal } from 'solid-js';

import {
  createSolidTable,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
} from '@tanstack/solid-table';
import type {
  ColumnFiltersState,
  SortingState,
  RowSelectionState,
  PaginationState,
  ColumnDef,
} from '@tanstack/solid-table';
import type { InstanceFile } from '@/entities/instances';
import { CONTENT_TABLE_COLUMNS } from './constants';

export interface CreateContentTableProps {
  headerActions: Component<{
    allRowsSelected?: boolean;
    someRowsSelected?: boolean;
    refetch?: () => void;
  }>;
  contentActions: Component<{ file: InstanceFile }>;
  data: Accessor<InstanceFile[]>;
  refetch: () => void;
  isLoading: Accessor<boolean>;
}

export const createContentTable = (props: CreateContentTableProps) => {
  const columns = createMemo(() => {
    const actionsColumn: ColumnDef<InstanceFile> = {
      id: 'actions',
      header: (rowProps) => (
        <div class='flex justify-end'>
          <props.headerActions
            allRowsSelected={rowProps.table.getIsAllPageRowsSelected()}
            someRowsSelected={rowProps.table.getIsSomePageRowsSelected()}
            refetch={props.refetch}
          />
        </div>
      ),
      cell: (rowProps) => (
        <div class='flex justify-end'>
          <props.contentActions file={rowProps.cell.row.original} />
        </div>
      ),
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
    pageIndex: 1,
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
