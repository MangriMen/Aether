import type { Accessor } from 'solid-js';

import { createColumnHelper } from '@tanstack/solid-table';

import type { TFunction } from '@/shared/model';

import { CombinedTextField } from '@/shared/ui';

import { JavaVersionStatusBadge } from '../ui/JavaPane/JavaVersionStatusBadge';

export interface JavaVersion {
  majorVersion: string;
  path: string | undefined;
}

export type JavaTestStatus = 'testing' | 'valid' | 'error' | 'version-mismatch';

export const javaVersionsColumnHelper = createColumnHelper<JavaVersion>();

interface CreateJavaVersionColumnsProps {
  t: TFunction;
  isInstalling?: Accessor<boolean>;
  onInstallRecommended?: (version: string) => void;
}

export const createJavaVersionColumns = (
  props: CreateJavaVersionColumnsProps,
) => [
  javaVersionsColumnHelper.accessor('majorVersion', {
    maxSize: 80,
    header: () => {
      return props.t('javaVersion.version');
    },
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
    header: () => {
      return props.t('javaVersion.path');
    },
    cell: (props) => (
      <CombinedTextField
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
    header: () => {
      return 'Status';
    },
    cell: (cellProps) => {
      return (
        <JavaVersionStatusBadge
          majorVersion={cellProps.row.original.majorVersion}
          path={cellProps.row.original.path}
          disabled={props.isInstalling?.()}
        />
      );
    },
  }),

  javaVersionsColumnHelper.display({
    id: 'actions',
    maxSize: 80,
    cell: () => {
      return 1;
    },
  }),
];
