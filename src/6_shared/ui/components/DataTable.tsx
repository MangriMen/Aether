import { createMemo, For, Show } from 'solid-js';
import type { ColumnDef, Table as SolidTable } from '@tanstack/solid-table';
import {
  createSolidTable,
  flexRender,
  getCoreRowModel,
} from '@tanstack/solid-table';

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/shared/ui';

type DataTableProps<TData, TValue> = {
  columns: ColumnDef<TData, TValue>[];
  isLoading?: boolean;
  noContentPlaceholder?: string;
} & (
  | { data: TData[]; columnsCount?: never; table?: never }
  | { data?: never; columnsCount?: number; table?: SolidTable<TData> }
);

export const DataTable = <TData, TValue>(
  props: DataTableProps<TData, TValue>,
) => {
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
        <TableHeader class='sticky top-0 z-10 bg-background'>
          <For each={table().getHeaderGroups()}>
            {(headerGroup) => (
              <TableRow>
                <For each={headerGroup.headers}>
                  {(header) => (
                    <TableHead colSpan={header.colSpan}>
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
                <TableRow data-state={row.getIsSelected() && 'selected'}>
                  <For each={row.getVisibleCells()}>
                    {(cell) => (
                      <TableCell>
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
