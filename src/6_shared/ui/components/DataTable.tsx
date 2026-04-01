import type { ColumnDef, Table as SolidTable } from '@tanstack/solid-table';

import {
  createSolidTable,
  flexRender,
  getCoreRowModel,
} from '@tanstack/solid-table';
import { createMemo, For, Show } from 'solid-js';

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/shared/ui';

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

  return (
    <div class='relative w-full overflow-auto rounded-md border'>
      <Table disableWrapper>
        <TableHeader class='sticky top-0 z-10 bg-secondary'>
          <For each={table().getHeaderGroups()}>
            {(headerGroup) => (
              <TableRow class='group'>
                <For each={headerGroup.headers}>
                  {(header) => (
                    <TableHead
                      colSpan={header.colSpan}
                      style={{
                        width: `${header.getSize()}px`,
                        'max-width': header.column.columnDef.maxSize
                          ? `${header.column.columnDef.maxSize}px`
                          : undefined,
                        'min-width': header.column.columnDef.minSize
                          ? `${header.column.columnDef.minSize}px`
                          : undefined,
                      }}
                    >
                      <Show when={!header.isPlaceholder}>
                        {flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                      </Show>
                    </TableHead>
                  )}
                </For>
              </TableRow>
            )}
          </For>
        </TableHeader>
        <TableBody>
          <Show
            when={table().getRowModel().rows?.length}
            fallback={
              <TableRow>
                <TableCell
                  colSpan={props.columns.length}
                  class='h-24 text-center'
                >
                  {props.noContentPlaceholder}
                </TableCell>
              </TableRow>
            }
          >
            <For each={table().getRowModel().rows}>
              {(row) => (
                <TableRow
                  class='group bg-secondary-dark/80'
                  data-state={row.getIsSelected() && 'selected'}
                >
                  <For each={row.getVisibleCells()}>
                    {(cell) => (
                      <TableCell
                        style={{
                          width: `${cell.column.getSize()}px`,
                          'max-width': cell.column.columnDef.maxSize
                            ? `${cell.column.columnDef.maxSize}px`
                            : undefined,
                          'min-width': cell.column.columnDef.minSize
                            ? `${cell.column.columnDef.minSize}px`
                            : undefined,
                        }}
                      >
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext(),
                        )}
                      </TableCell>
                    )}
                  </For>
                </TableRow>
              )}
            </For>
          </Show>
        </TableBody>
      </Table>
    </div>
  );
};
