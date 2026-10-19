import type { Column, RowData } from '@tanstack/solid-table';
import type { JSX } from 'solid-js';

import IconMdiChevronDown from '~icons/mdi/chevron-down';

import type { AppTableFeatures } from '@/shared/ui-kit';

import {
  getContentFileDisplayTitle,
  getContentFileVersionSortValue,
} from '@/features/content';
import { useTranslation } from '@/shared/i18n';
import { Button, Checkbox } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { contentTableColumnHelper } from '../model/contentTableColumnHelper';
import { ContentNameCell } from './ContentNameCell';
import { ContentVersionCell } from './ContentVersionCell';

interface SortableHeaderProps<TData extends RowData> {
  // oxlint-disable-next-line @typescript-eslint/no-explicit-any
  column: Column<AppTableFeatures, TData, any>;
  label: JSX.Element;
}

const SortableHeader = <TData extends RowData>(
  props: SortableHeaderProps<TData>,
) => {
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
          class={cn('text-base transition-transform', {
            'rotate-180': props.column.getIsSorted() === 'asc',
          })}
        />
      )}
    >
      {props.label}
    </Button>
  );
};

export const createContentTableColumns = () =>
  contentTableColumnHelper.columns([
    contentTableColumnHelper.display({
      id: 'select',
      minSize: 40,
      maxSize: 40,
      size: 40,
      header: (props) => (
        <div class='flex items-center justify-center'>
          <Checkbox
            checked={props.table.getIsAllPageRowsSelected()}
            indeterminate={
              props.table.getIsSomePageRowsSelected() &&
              !props.table.getIsAllPageRowsSelected()
            }
            onChange={(value) => props.table.toggleAllRowsSelected(value)}
            aria-label='Select all'
          />
        </div>
      ),
      cell: (props) => (
        <div class='flex items-center justify-center'>
          <Checkbox
            checked={props.row.getIsSelected()}
            onChange={(value) => props.row.toggleSelected(value)}
            aria-label='Select row'
          />
        </div>
      ),
    }),

    contentTableColumnHelper.accessor(getContentFileDisplayTitle, {
      id: 'name',
      header: (props) => {
        const [{ t }] = useTranslation();
        return (
          <SortableHeader column={props.column} label={t('common.content')} />
        );
      },
      cell: (props) => <ContentNameCell file={props.row.original} />,
    }),

    contentTableColumnHelper.accessor(getContentFileVersionSortValue, {
      id: 'version',
      header: (props) => {
        const [{ t }] = useTranslation();
        return (
          <SortableHeader column={props.column} label={t('common.version')} />
        );
      },
      cell: (props) => <ContentVersionCell file={props.row.original} />,
    }),

    contentTableColumnHelper.accessor('contentType', {
      id: 'type',
      size: 100,
      maxSize: 100,
      header: (props) => {
        const [{ t }] = useTranslation();
        return (
          <SortableHeader column={props.column} label={t('common.type')} />
        );
      },
    }),
  ]);
