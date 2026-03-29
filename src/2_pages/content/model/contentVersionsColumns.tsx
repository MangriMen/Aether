import type { ColumnDef } from '@tanstack/solid-table';

import { createMemo, For } from 'solid-js';

import type { ContentVersion } from '@/entities/instances';

import { dayjs, formatRelativeTime, formatTime } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Badge, CombinedTooltip } from '@/shared/ui';

import { formatDownloads } from '../lib';

export const CONTENT_VERSIONS_TABLE_COLUMNS: ColumnDef<ContentVersion>[] = [
  {
    id: 'name',
    accessorKey: 'name',
    enableSorting: true,
    header: () => {
      const [{ t }] = useTranslation();

      return t(`contentVersion.name`);
    },
    cell: (props) => {
      const versionNumber = createMemo(() => props.row.original.versionNumber);

      return (
        <div class='flex flex-col'>
          <span>{versionNumber()}</span>
          <span class='text-muted-foreground'>
            {props.cell.getValue() as string}
          </span>
        </div>
      );
    },
  },
  {
    id: 'gameVersions',
    accessorKey: 'gameVersions',
    header: () => {
      const [{ t }] = useTranslation();

      return t(`contentVersion.gameVersion`);
    },
    cell: (props) => {
      const value = createMemo(() => {
        return props.getValue() as string[];
      });

      return (
        <div class='flex flex-wrap gap-1 '>
          <For each={value()}>
            {(version) => <Badge variant='secondary'>{version}</Badge>}
          </For>
        </div>
      );
    },
  },
  {
    id: 'loaders',
    accessorKey: 'loaders',
    header: () => {
      const [{ t }] = useTranslation();

      return t(`contentVersion.platforms`);
    },
    cell: (props) => {
      const value = createMemo(() => {
        return props.getValue() as string[];
      });

      return (
        <div class='flex flex-wrap gap-1'>
          <For each={value()}>
            {(version) => <Badge variant='secondary'>{version}</Badge>}
          </For>
        </div>
      );
    },
  },
  {
    id: 'datePublished',
    accessorKey: 'datePublished',
    header: () => {
      const [{ t }] = useTranslation();

      return t(`contentVersion.published`);
    },
    cell: (props) => {
      const time = createMemo(() => {
        const date = dayjs(props.getValue() as string);

        return {
          time: formatTime(date),
          relative: formatRelativeTime(date),
        };
      });

      return (
        <CombinedTooltip label={time().time} as='span'>
          {time().relative}
        </CombinedTooltip>
      );
    },
  },
  {
    id: 'downloads',
    accessorKey: 'downloads',
    header: () => {
      const [{ t }] = useTranslation();

      return t(`contentVersion.downloads`);
    },
    cell: (props) => {
      const [{ locale }] = useTranslation();

      const formatted = createMemo(() => {
        const raw = props.getValue() as string;

        const value = Number(raw);
        if (Number.isNaN(value)) {
          return '?';
        }

        return formatDownloads(value, locale());
      });

      return <>{formatted()}</>;
    },
  },
];
