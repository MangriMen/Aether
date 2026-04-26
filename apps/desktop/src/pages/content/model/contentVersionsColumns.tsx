import { createColumnHelper } from '@tanstack/solid-table';
import { createMemo } from 'solid-js';

import type { ContentVersion } from '@/entities/instances';

import { dayjs, formatRelativeTime, formatTime } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

import { formatDownloads } from '../lib';
import { ContentVersionBadgeList } from '../ui/ContentVersionBadgeList';
import { ContentVersionTypeBadge } from '../ui/ContentVersionTypeBadge';

export const contentVersionsColumnHelper = createColumnHelper<ContentVersion>();

export const CONTENT_VERSIONS_TABLE_COLUMNS = [
  contentVersionsColumnHelper.display({
    id: 'versionType',
    size: 52,
    minSize: 52,
    maxSize: 52,
    cell: (props) => (
      <ContentVersionTypeBadge type={props.row.original.versionType} />
    ),
  }),

  contentVersionsColumnHelper.accessor('name', {
    header: () => {
      const [{ t }] = useTranslation();

      return t(`contentVersion.name`);
    },
    cell: (props) => {
      const versionNumber = createMemo(() => props.row.original.versionNumber);

      return (
        <div class='flex flex-col'>
          <span class='text-base font-medium'>{versionNumber()}</span>
          <span class='text-muted-foreground'>{props.cell.getValue()}</span>
        </div>
      );
    },
  }),

  contentVersionsColumnHelper.accessor('gameVersions', {
    header: () => {
      const [{ t }] = useTranslation();
      return t(`contentVersion.gameVersion`);
    },
    cell: (info) => <ContentVersionBadgeList items={info.getValue()} />,
  }),

  contentVersionsColumnHelper.accessor('loaders', {
    header: () => {
      const [{ t }] = useTranslation();
      return t(`contentVersion.platforms`);
    },
    cell: (info) => <ContentVersionBadgeList items={info.getValue()} />,
  }),

  contentVersionsColumnHelper.accessor('datePublished', {
    header: () => {
      const [{ t }] = useTranslation();
      return t(`contentVersion.published`);
    },
    cell: (info) => {
      const time = createMemo(() => {
        const date = dayjs(info.getValue());

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
  }),

  contentVersionsColumnHelper.accessor('downloads', {
    header: () => {
      const [{ t }] = useTranslation();
      return t(`contentVersion.downloads`);
    },
    cell: (info) => {
      const [{ locale }] = useTranslation();
      const formatted = createMemo(() =>
        formatDownloads(info.getValue(), locale()),
      );

      return <span>{formatted()}</span>;
    },
  }),
];
