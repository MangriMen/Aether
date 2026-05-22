import type {
  Column,
  ColumnDef,
  HeaderGroup,
  Row,
  Table as SolidTable,
} from '@tanstack/solid-table';
import type { Accessor, ComponentProps, JSX } from 'solid-js';

import {
  createSolidTable,
  flexRender,
  getCoreRowModel,
} from '@tanstack/solid-table';
import { createMemo, For, Show, Switch, Match, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { Skeleton } from './Skeleton';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from './Table';

const getSharedColumnsStyles = <TData,>(
  column: Column<TData, unknown>,
): JSX.CSSProperties => {
  const { stretch, center } = column.columnDef.meta ?? {};
  const { maxSize, minSize } = column.columnDef;

  if (stretch) {
    return {
      'text-align': center ? 'center' : undefined,
      width: '100%',
      'min-width': `${minSize ?? 150}px`,
      'max-width': '0px',
    };
  }

  return {
    'text-align': center ? 'center' : undefined,
    'max-width': maxSize ? `${maxSize}px` : undefined,
    'min-width': minSize ? `${minSize}px` : undefined,
  };
};

const createDefaultTable = <TData,>(
  data: Accessor<TData[] | undefined>,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  columns: Accessor<ColumnDef<TData, any>[]>,
) => {
  return createSolidTable({
    get data() {
      return data() ?? [];
    },
    get columns() {
      return columns();
    },
    getCoreRowModel: getCoreRowModel(),
  });
};

type BaseDataTableProps = {
  class?: string;
  tableClass?: string;
  isLoading?: boolean;
  noContentPlaceholder?: string;
};

type DataTableProps<TData> = BaseDataTableProps &
  (
    | {
        data: TData[];
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        columns: ColumnDef<TData, any>[];
        table?: never;
      }
    | {
        table: SolidTable<TData>;
        data?: never;
        columns?: never;
      }
  );

export const DataTable = <TData,>(
  props: ComponentProps<'div'> & DataTableProps<TData>,
) => {
  const [local, others] = splitProps(props, [
    'class',
    'tableClass',
    'columns',
    'isLoading',
    'noContentPlaceholder',
    'data',
    'table',
  ]);

  const [{ t }] = useTranslation();

  const table = createMemo(() => {
    if (local.table) {
      return local.table;
    }

    const data = local.data;
    const columns = local.columns;

    if (!data || !columns) {
      throw new Error("If table not provided, 'data' and 'columns' must be");
    }

    return createDefaultTable(
      () => data,
      () => columns,
    );
  });

  const columnsData = createMemo(() => {
    const visibleLeafColumns = table().getVisibleLeafColumns();
    const columnStyles: Record<string, JSX.CSSProperties> = {};

    visibleLeafColumns.forEach((column) => {
      columnStyles[column.id] = getSharedColumnsStyles(column);
    });

    return {
      columnStyles,
      visibleColumns: visibleLeafColumns,
      columnsCount: visibleLeafColumns.length,
    };
  });

  return (
    <div
      class={cn('relative w-full overflow-auto rounded-md border', local.class)}
      {...others}
    >
      <Table class={local.tableClass} disableWrapper>
        <TableHeader class='sticky top-0 z-10 bg-popover'>
          <For each={table().getHeaderGroups()}>
            {(headerGroup) => (
              <DataTableHeaderRow
                headerGroup={headerGroup}
                cellStyles={columnsData().columnStyles}
              />
            )}
          </For>
        </TableHeader>
        <TableBody>
          <Switch>
            <Match when={Boolean(local.isLoading)}>
              <TableSkeleton
                rowsCount={6}
                visibleColumns={columnsData().visibleColumns}
                columnStyles={columnsData().columnStyles}
              />
            </Match>

            <Match when={!local.isLoading && table().getRowCount() === 0}>
              <TableStatusMessage colSpan={columnsData().columnsCount}>
                {local.noContentPlaceholder ?? t('common.noData')}
              </TableStatusMessage>
            </Match>

            <Match when>
              <For each={table().getRowModel().rows}>
                {(row) => (
                  <DataTableRow
                    row={row}
                    cellStyles={columnsData().columnStyles}
                  />
                )}
              </For>
            </Match>
          </Switch>
        </TableBody>
      </Table>
    </div>
  );
};

interface DataTableHeaderRowProps<TData> {
  headerGroup: HeaderGroup<TData>;
  cellStyles: Record<string, JSX.CSSProperties>;
}

export const DataTableHeaderRow = <TData,>(
  props: DataTableHeaderRowProps<TData>,
) => {
  return (
    <TableRow class='group'>
      <For each={props.headerGroup.headers}>
        {(header) => (
          <TableHead
            colSpan={header.colSpan}
            style={props.cellStyles[header.column.id]}
          >
            <Show when={!header.isPlaceholder}>
              {flexRender(header.column.columnDef.header, header.getContext())}
            </Show>
          </TableHead>
        )}
      </For>
    </TableRow>
  );
};

interface DataTableRowProps<TData> {
  row: Row<TData>;
  cellStyles: Record<string, JSX.CSSProperties>;
}

const DataTableRow = <TData,>(props: DataTableRowProps<TData>) => {
  return (
    <TableRow
      class='group bg-secondary/card'
      data-state={props.row.getIsSelected() && 'selected'}
    >
      <For each={props.row.getVisibleCells()}>
        {(cell) => (
          <TableCell style={props.cellStyles[cell.column.id]}>
            {flexRender(cell.column.columnDef.cell, cell.getContext())}
          </TableCell>
        )}
      </For>
    </TableRow>
  );
};

interface TableStatusMessageProps {
  colSpan: number;
}

const TableStatusMessage = (
  props: ComponentProps<'tr'> & TableStatusMessageProps,
) => {
  const [local, others] = splitProps(props, ['colSpan', 'children']);

  return (
    <TableRow {...others}>
      <TableCell colSpan={local.colSpan} class='h-24 text-center'>
        {local.children}
      </TableCell>
    </TableRow>
  );
};

interface TableSkeletonProps<TData> {
  rowsCount: number;
  visibleColumns: Column<TData, unknown>[];
  columnStyles: Record<string, JSX.CSSProperties>;
}

const TableSkeleton = <TData,>(props: TableSkeletonProps<TData>) => {
  const rowIndices = createMemo(() => new Array(props.rowsCount).fill(0));

  return (
    <For each={rowIndices()}>
      {() => (
        <TableRow class='bg-secondary/card'>
          <For each={props.visibleColumns}>
            {(column) => (
              <TableCell style={props.columnStyles[column.id]}>
                <Skeleton height={32} radius={6} />
              </TableCell>
            )}
          </For>
        </TableRow>
      )}
    </For>
  );
};
