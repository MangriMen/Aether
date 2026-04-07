import type { Column } from '@tanstack/solid-table';
import type { JSX } from 'solid-js';

import { createColumnHelper } from '@tanstack/solid-table';
import IconMdiChevronDown from '~icons/mdi/chevron-down';
import { Show } from 'solid-js';

import type { ContentFile } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Checkbox } from '@/shared/ui';

export const contentTableColumnHelper = createColumnHelper<ContentFile>();

interface SortableHeaderProps<TData> {
  column: Column<TData>;
  label: JSX.Element;
}

const SortableHeader = <TData,>(props: SortableHeaderProps<TData>) => {
  return (
    <Button
      class='p-0'
      variant='ghost'
      size='sm'
      onClick={() =>
        props.column.toggleSorting(props.column.getIsSorted() === 'asc')
      }
      trailingIcon={() => (
        <IconMdiChevronDown
          class={cn('transition-transform text-base', {
            'rotate-180': props.column.getIsSorted() === 'asc',
          })}
        />
      )}
    >
      {props.label}
    </Button>
  );
};

export const CONTENT_TABLE_COLUMNS = [
  contentTableColumnHelper.display({
    id: 'select',
    minSize: 40,
    maxSize: 40,
    size: 40,
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
  }),

  contentTableColumnHelper.accessor((row) => row.name ?? row.fileName, {
    id: 'name',
    header: (props) => {
      const [{ t }] = useTranslation();
      return <SortableHeader column={props.column} label={t('common.name')} />;
    },
    cell: (props) => {
      const original = () => props.row.original;

      return (
        <span
          class={cn('inline-flex flex-col', {
            'text-muted-foreground': original().disabled,
          })}
        >
          <Show when={original().name}>{(name) => <span>{name()}</span>}</Show>
          <span
            class={cn({
              'text-muted-foreground': !!original().name,
            })}
          >
            {original().fileName}
          </span>
        </span>
      );
    },
  }),

  contentTableColumnHelper.accessor('contentType', {
    id: 'type',
    size: 100,
    maxSize: 100,
    header: (props) => {
      const [{ t }] = useTranslation();
      return <SortableHeader column={props.column} label={t('common.type')} />;
    },
  }),
];
