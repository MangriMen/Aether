import { createColumnHelper } from '@tanstack/solid-table';
import { createMemo, type Accessor } from 'solid-js';

import type { TFunction } from '@/shared/model';

import { JavaVersionActions } from '../ui/JavaPane/JavaVersionActions';
import { JavaVersionPath } from '../ui/JavaPane/JavaVersionPath';
import { JavaVersionStatusBadge } from '../ui/JavaPane/JavaVersionStatusBadge';

export interface JavaVersion {
  majorVersion: string;
  path: string | undefined;
}

export type JavaTestStatus =
  | 'idle'
  | 'testing'
  | 'valid'
  | 'error'
  | 'version-mismatch';

export const javaVersionsColumnHelper = createColumnHelper<JavaVersion>();

interface CreateJavaVersionColumnsProps {
  t: TFunction;
  isInstalling?: Accessor<boolean>;
  onInstallRecommended?: (version: string) => void;
}

export const createJavaVersionColumns = ({
  t,
  isInstalling,
  onInstallRecommended,
}: CreateJavaVersionColumnsProps) => {
  const columns = createMemo(() => [
    javaVersionsColumnHelper.accessor('majorVersion', {
      maxSize: 80,
      header: () => t('javaVersion.version'),
      cell: (props) => {
        return <span>{`Java ${props.cell.renderValue() ?? '—'}`}</span>;
      },
    }),

    javaVersionsColumnHelper.accessor('path', {
      header: () => t('javaVersion.path'),
      cell: (cellProps) => (
        <JavaVersionPath
          majorVersion={cellProps.row.original.majorVersion}
          value={cellProps.getValue()}
        />
      ),
      meta: {
        stretch: true,
      },
    }),

    javaVersionsColumnHelper.display({
      id: 'status',
      minSize: 150,
      header: () => t('javaVersion.status'),
      cell: (cellProps) => (
        <JavaVersionStatusBadge
          class='whitespace-nowrap'
          majorVersion={cellProps.row.original.majorVersion}
          path={cellProps.row.original.path}
          disabled={isInstalling?.()}
        />
      ),
      meta: {
        center: true,
      },
    }),

    javaVersionsColumnHelper.display({
      id: 'actions',
      maxSize: 80,
      cell: (cellProps) => (
        <JavaVersionActions
          isInstalling={isInstalling?.()}
          onInstallRecommended={() =>
            onInstallRecommended?.(cellProps.row.original.majorVersion)
          }
        />
      ),
    }),
  ]);

  return columns;
};
