import type {
  ColumnFiltersState,
  SortingState,
  RowSelectionState,
  PaginationState,
} from '@tanstack/solid-table';
import type { Accessor, Component } from 'solid-js';

import { createTable } from '@tanstack/solid-table';
import { createEffect, createMemo, createSignal, on } from 'solid-js';

import type { ContentFile } from '@/features/content';

import { appTableFeatures } from '@/shared/ui-kit';

import { contentTableColumnHelper } from '../model/contentTableColumnHelper';
import { createContentTableColumns } from '../ui/createContentTableColumns';

export interface CreateContentTableProps {
  headerActions: Component<{
    refetch?: () => void;
  }>;
  contentActions: Component<{ file: ContentFile }>;
  data: Accessor<ContentFile[]>;
  refetch: () => void;
  isLoading: Accessor<boolean>;
  /** The selection belongs to one instance, so it is dropped when this changes. */
  instanceId?: Accessor<string>;
}

export const createContentTable = (props: CreateContentTableProps) => {
  const [rowSelection, setRowSelection] = createSignal<RowSelectionState>({});
  const clearSelection = () => setRowSelection({});

  const columns = createMemo(() => {
    const actionsColumn = contentTableColumnHelper.display({
      id: 'actions',
      size: 116,
      maxSize: 116,
      header: () => (
        <div class='flex justify-end'>
          <props.headerActions refetch={props.refetch} />
        </div>
      ),
      cell: (cellProps) => (
        <div class='flex justify-end'>
          {/* Используем сокращенный путь props.row.original */}
          <props.contentActions file={cellProps.row.original} />
        </div>
      ),
    });

    return [...createContentTableColumns(), actionsColumn];
  });

  const [sorting, setSorting] = createSignal<SortingState>([
    { desc: true, id: 'name' },
  ]);
  const [columnFilters, setColumnFilters] = createSignal<ColumnFiltersState>(
    [],
  );
  const [pagination, setPagination] = createSignal<PaginationState>({
    pageIndex: 0,
    pageSize: 20,
  });

  // Rows hidden by a filter stay in the selection otherwise, and a bulk action would still touch them.
  const handleColumnFiltersChange: typeof setColumnFilters = (updater) => {
    setColumnFilters(updater);
    clearSelection();
  };

  createEffect(
    on(
      () => props.instanceId?.(),
      () => clearSelection(),
      { defer: true },
    ),
  );

  const table = createTable({
    features: appTableFeatures,
    get data() {
      return props.data();
    },
    get columns() {
      return columns();
    },
    onSortingChange: setSorting,
    // Selection is keyed by path, so it survives the data coming back in another order.
    getRowId: (row) => row.contentPath,
    onColumnFiltersChange: handleColumnFiltersChange,
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

  return table;
};
