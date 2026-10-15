import type { Column, RowData } from '@tanstack/solid-table';
import type { JSX } from 'solid-js';

import { A, useParams } from '@solidjs/router';
import { createColumnHelper } from '@tanstack/solid-table';
import IconMdiChevronDown from '~icons/mdi/chevron-down';
import { Match, Show, Switch } from 'solid-js';

import type { ContentFile } from '@/entities/instances';
import type { AppTableFeatures } from '@/shared/model';

import {
  getContentFileAuthor,
  getContentFileDisplayTitle,
  getContentFileIconSrc,
  getContentFileProjectHref,
  getContentFileVersionNumber,
  getContentFileVersionSortValue,
  getContentFileVersionState,
  isLocalContentFile,
  InstanceIcon,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Checkbox } from '@/shared/ui';

export const contentTableColumnHelper = createColumnHelper<
  AppTableFeatures,
  ContentFile
>();

interface SortableHeaderProps<TData extends RowData> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
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

export const CONTENT_TABLE_COLUMNS = contentTableColumnHelper.columns([
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
    cell: (props) => {
      const [{ t }] = useTranslation();
      const original = () => props.row.original;

      return (
        <span
          class={cn('inline-flex items-center gap-2', {
            'text-muted-foreground': original().disabled,
          })}
        >
          <InstanceIcon
            class='size-8'
            src={getContentFileIconSrc(original())}
          />
          <span class='inline-flex flex-col'>
            <span>{getContentFileDisplayTitle(original())}</span>
            <Show
              when={!isLocalContentFile(original())}
              fallback={
                <span class='text-muted-foreground'>{t('common.local')}</span>
              }
            >
              <Show when={getContentFileAuthor(original())}>
                {(author) => (
                  <span class='text-muted-foreground'>{author()}</span>
                )}
              </Show>
            </Show>
          </span>
        </span>
      );
    },
  }),

  contentTableColumnHelper.accessor(getContentFileVersionSortValue, {
    id: 'version',
    header: (props) => {
      const [{ t }] = useTranslation();
      return (
        <SortableHeader column={props.column} label={t('common.version')} />
      );
    },
    cell: (props) => {
      const [{ t }] = useTranslation();
      const params = useParams();
      const original = () => props.row.original;

      const href = () => getContentFileProjectHref(original(), params.id);

      return (
        <span
          class={cn('inline-flex flex-col', {
            'text-muted-foreground': original().disabled,
          })}
        >
          <Switch>
            <Match when={getContentFileVersionState(original()) === 'known'}>
              <Show
                when={href()}
                fallback={
                  <span>{getContentFileVersionNumber(original())}</span>
                }
              >
                {(link) => (
                  <A href={link()} class='hover:underline'>
                    {getContentFileVersionNumber(original())}
                  </A>
                )}
              </Show>
            </Match>
            <Match when={getContentFileVersionState(original()) === 'pending'}>
              <span class='text-muted-foreground'>{t('common.loading')}</span>
            </Match>
            <Match when>
              <span>{t('common.unknown')}</span>
            </Match>
          </Switch>
          <span class='text-muted-foreground'>{original().fileName}</span>
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
]);
