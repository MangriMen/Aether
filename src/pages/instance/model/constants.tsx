import type { ColumnDef } from '@tanstack/solid-table';

import { Icon } from '@iconify-icon/solid';

import MdiChevronDownIcon from '@iconify/icons-mdi/chevron-down';
import { Button, Checkbox } from '@/shared/ui';
import type { InstanceFile } from '@/entities/instances';
import { cn } from '@/shared/lib';

export const CONTENT_TABLE_COLUMNS: ColumnDef<InstanceFile>[] = [
  {
    id: 'select',
    header: (props) => (
      <div class='flex items-center justify-center p-0'>
        <Checkbox
          checked={props.table.getIsAllPageRowsSelected()}
          indeterminate={props.table.getIsSomePageRowsSelected()}
          onChange={(value) => props.table.toggleAllRowsSelected(value)}
          aria-label='Select all'
        />
      </div>
    ),
    cell: (props) => (
      <div class='flex items-center justify-center p-0'>
        <Checkbox
          checked={props.row.getIsSelected()}
          onChange={(value) => props.row.toggleSelected(value)}
          aria-label='Select row'
        />
      </div>
    ),
  },
  {
    accessorKey: 'fileName',
    id: 'name',
    header: (props) => (
      <Button
        variant='ghost'
        size='sm'
        onClick={() =>
          props.column.toggleSorting(props.column.getIsSorted() === 'asc')
        }
        trailingIcon={
          <Icon
            class={cn('text-lg transition-transform', {
              'rotate-180': props.column.getIsSorted() === 'asc',
            })}
            icon={MdiChevronDownIcon}
          />
        }
        children='Name'
      />
    ),
    cell: (props) => (
      <span
        class={cn({
          'text-muted-foreground': props.cell.row.original.disabled,
        })}
      >
        {props.cell.row.original.fileName.replace('.disabled', '')}
      </span>
    ),
  },
  {
    accessorKey: 'contentType',
    id: 'type',
    header: (props) => (
      <Button
        variant='ghost'
        size='sm'
        onClick={() =>
          props.column.toggleSorting(props.column.getIsSorted() === 'asc')
        }
        trailingIcon={
          <Icon
            class={cn('text-lg transition-transform', {
              'rotate-180': props.column.getIsSorted() === 'asc',
            })}
            icon={MdiChevronDownIcon}
          />
        }
        children='Type'
      />
    ),
  },
];
