import { Show } from 'solid-js';
import type { ColumnDef } from '@tanstack/solid-table';

import { Icon } from '@iconify-icon/solid';

import MdiChevronDownIcon from '@iconify/icons-mdi/chevron-down';
import { Button, Checkbox } from '@/shared/ui';
import type { InstanceFile } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

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
    id: 'name',
    accessorKey: 'name',
    header: (props) => {
      const [{ t }] = useTranslation();

      return (
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
          children={<>{t('common.name')}</>}
        />
      );
    },
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
  },
  {
    id: 'type',
    accessorKey: 'contentType',
    header: (props) => {
      const [{ t }] = useTranslation();

      return (
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
          children={<>{t('common.type')}</>}
        />
      );
    },
  },
];
