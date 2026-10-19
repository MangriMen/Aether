import type {
  Column,
  ColumnDef,
  HeaderGroup,
  Row,
  RowData,
  SolidTable,
} from '@tanstack/solid-table';
import type { Accessor, ComponentProps, JSX } from 'solid-js';

import { createTable, FlexRender } from '@tanstack/solid-table';
import { createMemo, For, Show, Switch, Match, splitProps } from 'solid-js';

import { cn } from '@/shared/utils';

import type { AppTableFeatures } from '../model/tableFeatures';

import { appTableFeatures } from '../model/tableFeatures';
import { Skeleton } from './Skeleton';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from './Table';
import { useUiKitLabels } from './UiKitLabels';

// Tables passed to DataTable may enable more features than this set.
type DataTableFeatures = AppTableFeatures;

const getSharedColumnsStyles = <TData extends RowData>(
  column: Column<DataTableFeatures, TData, unknown>,
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

const createDefaultTable = <TData extends RowData>(
  data: Accessor<TData[] | undefined>,
  // oxlint-disable-next-line @typescript-eslint/no-explicit-any
  columns: Accessor<ColumnDef<DataTableFeatures, TData, any>[]>,
) => {
  return createTable({
    features: appTableFeatures,
    manualPagination: true,
    get data() {
      return data() ?? [];
    },
    get columns() {
      return columns();
    },
  });
};

type BaseDataTableProps = {
  class?: string;
  tableClass?: string;
  isLoading?: boolean;
  noContentPlaceholder?: string;
};

type DataTableProps<TData extends RowData> = BaseDataTableProps &
  (
    | {
        data: TData[];
        // oxlint-disable-next-line @typescript-eslint/no-explicit-any
        columns: ColumnDef<DataTableFeatures, TData, any>[];
        table?: never;
      }
    | {
        table: SolidTable<DataTableFeatures, TData>;
        data?: never;
        columns?: never;
      }
  );

export const DataTable = <TData extends RowData>(
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

  const labels = useUiKitLabels();

  const getTable = () => {
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
  };

  const columnsData = createMemo(() => {
    const visibleLeafColumns = getTable().getVisibleLeafColumns();
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
      class={cn('rounded-md relative w-full overflow-auto border', local.class)}
      {...others}
    >
      <Table class={local.tableClass} disableWrapper>
        <TableHeader class='top-0 bg-popover sticky z-10'>
          <For each={getTable().getHeaderGroups()}>
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

            <Match when={!local.isLoading && getTable().getRowCount() === 0}>
              <TableStatusMessage colSpan={columnsData().columnsCount}>
                {local.noContentPlaceholder ?? labels.noData}
              </TableStatusMessage>
            </Match>

            <Match when>
              <For each={getTable().getRowModel().rows}>
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

interface DataTableHeaderRowProps<TData extends RowData> {
  headerGroup: HeaderGroup<DataTableFeatures, TData>;
  cellStyles: Record<string, JSX.CSSProperties>;
}

export const DataTableHeaderRow = <TData extends RowData>(
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
              <FlexRender header={header} />
            </Show>
          </TableHead>
        )}
      </For>
    </TableRow>
  );
};

interface DataTableRowProps<TData extends RowData> {
  row: Row<DataTableFeatures, TData>;
  cellStyles: Record<string, JSX.CSSProperties>;
}

const DataTableRow = <TData extends RowData>(
  props: DataTableRowProps<TData>,
) => {
  return (
    <TableRow
      class='group bg-secondary/card'
      data-state={props.row.getIsSelected() && 'selected'}
    >
      <For each={props.row.getVisibleCells()}>
        {(cell) => (
          <TableCell style={props.cellStyles[cell.column.id]}>
            <FlexRender cell={cell} />
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

interface TableSkeletonProps<TData extends RowData> {
  rowsCount: number;
  visibleColumns: Column<DataTableFeatures, TData, unknown>[];
  columnStyles: Record<string, JSX.CSSProperties>;
}

const TableSkeleton = <TData extends RowData>(
  props: TableSkeletonProps<TData>,
) => {
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
