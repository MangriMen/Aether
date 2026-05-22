import { createColumnHelper } from '@tanstack/solid-table';
import IconMdiCheck from '~icons/mdi/check';
import { createMemo, Show, type Accessor } from 'solid-js';

import type { TFunction } from '@/shared/model';

import { Button, CombinedTooltip } from '@/shared/ui';

import { JavaVersionActions } from '../ui/JavaPane/JavaVersionActions';
import { JavaVersionPath } from '../ui/JavaPane/JavaVersionPath';
import { JavaVersionStatusButton } from '../ui/JavaPane/JavaVersionStatusButton';

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
  onDetect?: (version: string) => void;
}

export const createJavaVersionColumns = ({
  t,
  isInstalling,
  onInstallRecommended,
  onDetect,
}: CreateJavaVersionColumnsProps) => {
  const columns = createMemo(() => [
    javaVersionsColumnHelper.accessor('majorVersion', {
      maxSize: 80,
      header: () => t('javaVersion.version'),
      cell: (props) => {
        return (
          <span>
            {t('javaVersion.versionWithNumber', {
              majorVersion: props.getValue(),
            })}
          </span>
        );
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
      minSize: 160,
      header: () => t('javaVersion.status'),
      cell: (cellProps) => (
        <JavaVersionStatusButton
          class='w-full whitespace-nowrap'
          majorVersion={cellProps.row.original.majorVersion}
          path={cellProps.row.original.path}
          disabled={isInstalling?.()}
        />
      ),
      meta: {
        center: true,
        // stretch: true,
      },
    }),

    javaVersionsColumnHelper.display({
      id: 'actions',
      maxSize: 80,
      cell: (cellProps) => {
        const majorVersion = () => cellProps.row.original.majorVersion;

        return (
          <JavaVersionActions
            isInstalling={isInstalling?.()}
            onInstallRecommended={() => onInstallRecommended?.(majorVersion())}
            onDetect={() => onDetect?.(majorVersion())}
          />
        );
      },
    }),
  ]);

  return columns;
};

interface CreateDetectedJavaVersionColumnsProps {
  t: TFunction;
  currentVersion?: Accessor<JavaVersion | undefined>;
  onSelect?: (javaVersion: JavaVersion) => void;
}

export const createDetectedJavaVersionColumns = ({
  t,
  currentVersion,
  onSelect,
}: CreateDetectedJavaVersionColumnsProps) => {
  const columns = createMemo(() => [
    javaVersionsColumnHelper.accessor('majorVersion', {
      header: () => t('javaVersion.version'),
    }),

    javaVersionsColumnHelper.accessor('path', {
      header: () => t('javaVersion.path'),
      cell: (cellProps) => (
        <CombinedTooltip label={cellProps.getValue()} as='div' class='truncate'>
          {cellProps.renderValue()}
        </CombinedTooltip>
      ),
      meta: {
        stretch: true,
      },
    }),

    javaVersionsColumnHelper.display({
      id: 'actions',
      minSize: 120,
      cell: (cellProps) => {
        const isSelected = () =>
          currentVersion?.()?.path === cellProps.row.original.path;

        return (
          <Button
            class='w-full'
            onClick={() => onSelect?.(cellProps.row.original)}
            disabled={isSelected()}
            leadingIcon={isSelected() ? IconMdiCheck : undefined}
          >
            <Show when={isSelected()} fallback={t('detectJava.select')}>
              {t('detectJava.selected')}
            </Show>
          </Button>
        );
      },
    }),
  ]);

  return columns;
};
