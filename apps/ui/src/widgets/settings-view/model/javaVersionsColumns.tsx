import { createColumnHelper } from '@tanstack/solid-table';
import { createMemo, type Accessor } from 'solid-js';

import type { TFunction } from '@/shared/model';

import { CombinedTextField } from '@/shared/ui';

import { JavaVersionActions } from '../ui/JavaPane/JavaVersionActions';
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
        return (
          <span>
            <span>Java</span>
            &nbsp;
            {props.cell.renderValue()}
          </span>
        );
      },
    }),

    javaVersionsColumnHelper.accessor('path', {
      header: () => t('javaVersion.path'),
      cell: (props) => (
        <CombinedTextField
          class='h-8'
          value={props.cell.getValue()}
          // onChange={handlePathChange}
          inputProps={{ type: 'text', placeholder: '/path/to/java' }}
          readOnly
          //// readOnly={local.onDetect === undefined || local.onBrowse === undefined}
        />
      ),
      meta: {
        stretch: true,
      },
    }),

    javaVersionsColumnHelper.display({
      id: 'status',
      header: () => t('javaVersion.status'),
      cell: (cellProps) => {
        return (
          <JavaVersionStatusBadge
            majorVersion={cellProps.row.original.majorVersion}
            path={cellProps.row.original.path}
            disabled={isInstalling?.()}
          />
        );
      },
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
