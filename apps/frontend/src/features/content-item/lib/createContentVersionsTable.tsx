import type {
  ColumnFiltersState,
  PaginationState,
  RowSelectionState,
  SortingState,
} from '@tanstack/solid-table';
import type { Accessor } from 'solid-js';

import { createTable } from '@tanstack/solid-table';
import { createMemo, createSignal } from 'solid-js';

import type { ContentType, ContentVersion } from '@/features/content';

import { appTableFeatures } from '@/shared/ui-kit';

import { ContentVersionActions } from '../ui/ContentVersionActions';
import {
  CONTENT_VERSIONS_TABLE_COLUMNS,
  contentVersionsColumnHelper,
} from '../ui/ContentVersionsColumns';

export interface ContentVersionsTableProps {
  data: Accessor<ContentVersion[]>;
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

  const columns = createMemo(() => {
    return [
      ...CONTENT_VERSIONS_TABLE_COLUMNS,
      contentVersionsColumnHelper.display({
        id: 'actions',
        cell: (cellProps) => (
          <ContentVersionActions
            class='justify-end'
            version={cellProps.row.original}
            contentType={props.contentType()}
          />
        ),
      }),
    ];
  });

  const table = createTable({
    features: appTableFeatures,
    get data() {
      return props.data();
    },
    get columns() {
      return columns();
    },
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

  return table;
};
