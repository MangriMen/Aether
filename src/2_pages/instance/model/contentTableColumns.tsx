import type { ColumnDef } from '@tanstack/solid-table';

import { Icon } from '@iconify-icon/solid';
import MdiChevronDownIcon from '@iconify/icons-mdi/chevron-down';
import { Show } from 'solid-js';

import type { InstanceFile } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Checkbox } from '@/shared/ui';

export const CONTENT_TABLE_COLUMNS: ColumnDef<InstanceFile>[] = [
  {
    cell: (props) => (
      <div class='flex items-center justify-center p-0'>
        <Checkbox
          aria-label='Select row'
          checked={props.row.getIsSelected()}
          onChange={(value) => props.row.toggleSelected(value)}
        />
      </div>
    ),
    header: (props) => (
      <div class='flex items-center justify-center p-0'>
        <Checkbox
          aria-label='Select all'
          checked={props.table.getIsAllPageRowsSelected()}
          indeterminate={props.table.getIsSomePageRowsSelected()}
          onChange={(value) => props.table.toggleAllRowsSelected(value)}
        />
      </div>
    ),
    id: 'select',
  },
  {
    accessorKey: 'name',
    cell: (props) => (
      <span
        class={cn('inline-flex flex-col', {
          'text-muted-foreground': props.cell.row.original.disabled,
        })}
      >
        <Show when={props.cell.row.original.name}>
          {(name) => <span> {name()}</span>}
        </Show>
        <span
          class={cn({
            'text-muted-foreground': !!props.cell.row.original.name,
          })}
        >
          {props.cell.row.original.fileName.replace('.disabled', '')}
        </span>
      </span>
    ),
    header: (props) => {
      const [{ t }] = useTranslation();

      return (
        <Button
          children={<>{t('common.name')}</>}
          onClick={() =>
            props.column.toggleSorting(props.column.getIsSorted() === 'asc')
          }
          size='sm'
          trailingIcon={
            <Icon
              class={cn('text-lg transition-transform', {
                'rotate-180': props.column.getIsSorted() === 'asc',
              })}
              icon={MdiChevronDownIcon}
            />
          }
          variant='ghost'
        />
      );
    },
    id: 'name',
  },
  {
    accessorKey: 'contentType',
    header: (props) => {
      const [{ t }] = useTranslation();

      return (
        <Button
          children={<>{t('common.type')}</>}
          onClick={() =>
            props.column.toggleSorting(props.column.getIsSorted() === 'asc')
          }
          size='sm'
          trailingIcon={
            <Icon
              class={cn('text-lg transition-transform', {
                'rotate-180': props.column.getIsSorted() === 'asc',
              })}
              icon={MdiChevronDownIcon}
            />
          }
          variant='ghost'
        />
      );
    },
    id: 'type',
  },
];
