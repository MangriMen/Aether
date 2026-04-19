import type {
  Column,
  ColumnDef,
  Header,
  HeaderGroup,
  Row,
  Table as SolidTable,
} from '@tanstack/solid-table';
import type { ComponentProps, JSX } from 'solid-js';

import {
  createSolidTable,
  flexRender,
  getCoreRowModel,
} from '@tanstack/solid-table';
import { createMemo, For, Show, Switch, Match, splitProps } from 'solid-js';

import { Skeleton } from './Skeleton';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from './Table';

type DataTableProps<TData> = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  columns: ColumnDef<TData, any>[];
  isLoading?: boolean;
  noContentPlaceholder?: string;
} & (
  | { data: TData[]; columnsCount?: never; table?: never }
  | { data?: never; columnsCount?: number; table?: SolidTable<TData> }
);

export const DataTable = <TData,>(props: DataTableProps<TData>) => {
  const table = createMemo(
    () =>
      props.table ??
      createSolidTable({
        get data() {
          return props.data ?? [];
        },
        get columns() {
          return props.columns;
        },
        getCoreRowModel: getCoreRowModel(),
      }),
  );

  const columnsData = createMemo(() => {
    const visibleLeafColumns = table().getVisibleLeafColumns();

    const columnStyles: Record<string, JSX.CSSProperties> = {};
    visibleLeafColumns.forEach((column) => {
      columnStyles[column.id] = getColumnStyles(column);
    });

    return {
      columnStyles,
      columnsCount: visibleLeafColumns.length,
    };
  });

  return (
    <div class='relative w-full overflow-auto rounded-md border'>
      <Table disableWrapper>
        <TableHeader class='sticky top-0 z-10 bg-secondary'>
          <For each={table().getHeaderGroups()}>
            {(headerGroup) => <DataTableHeaderRow headerGroup={headerGroup} />}
          </For>
        </TableHeader>
        <TableBody>
          <Switch>
            <Match when={Boolean(props.isLoading)}>
              <TableSkeleton
                rowsCount={6}
                columnStyles={columnsData().columnStyles}
              />
            </Match>

            <Match when={table().getRowCount() === 0}>
              <TableStatusMessage colSpan={columnsData().columnsCount}>
                {props.noContentPlaceholder}
              </TableStatusMessage>
            </Match>

            <Match when={true}>
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
  cellStyles?: JSX.CSSProperties;
}

export const DataTableHeaderRow = <TData,>(
  props: DataTableHeaderRowProps<TData>,
) => {
  return (
    <TableRow class='group'>
      <For each={props.headerGroup.headers}>
        {(header) => (
          <TableHead colSpan={header.colSpan} style={getHeaderStyles(header)}>
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
  cellStyles?: Record<string, JSX.CSSProperties>;
}

const DataTableRow = <TData,>(props: DataTableRowProps<TData>) => {
  return (
    <TableRow
      class='group bg-new-secondary/card'
      data-state={props.row.getIsSelected() && 'selected'}
    >
      <For each={props.row.getVisibleCells()}>
        {(cell) => (
          <TableCell style={props.cellStyles?.[cell.column.id]}>
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

interface TableSkeletonProps {
  rowsCount: number;
  columnStyles: Record<string, JSX.CSSProperties>;
}

const TableSkeleton = (props: TableSkeletonProps) => {
  const rows = createMemo(() => Array.from({ length: props.rowsCount }));
  const stylesList = createMemo(() => Object.values(props.columnStyles));

  return (
    <For each={rows()}>
      {() => (
        <TableRow class='bg-new-secondary/card'>
          <For each={stylesList()}>
            {(style) => (
              <TableCell style={style}>
                <Skeleton height={32} radius={6} />
              </TableCell>
            )}
          </For>
        </TableRow>
      )}
    </For>
  );
};

const getHeaderStyles = <TData, TValue>(
  header: Header<TData, TValue>,
): JSX.CSSProperties => {
  const def = header.column.columnDef;

  return {
    width: `${header.getSize()}px`,
    ...(def.maxSize && { 'max-width': `${def.maxSize}px` }),
    ...(def.minSize && { 'min-width': `${def.minSize}px` }),
  };
};

const getColumnStyles = <TData, TValue>(
  column: Column<TData, TValue>,
): JSX.CSSProperties => {
  const def = column.columnDef;

  return {
    width: `${column.getSize()}px`,
    ...(def.maxSize && { 'max-width': `${def.maxSize}px` }),
    ...(def.minSize && { 'min-width': `${def.minSize}px` }),
  };
};
