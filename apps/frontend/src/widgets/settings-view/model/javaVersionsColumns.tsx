import { createColumnHelper } from '@tanstack/solid-table';

import type { TFunction } from '@/shared/model';

import type { JavaVersion } from './javaVersion';

import { JavaVersionActions } from '../ui/JavaPane/JavaVersionActions';
import { JavaVersionPath } from '../ui/JavaPane/JavaVersionPath';
import { JavaVersionStatusButton } from '../ui/JavaPane/JavaVersionStatusButton';

export const javaVersionsColumnHelper = createColumnHelper<JavaVersion>();

interface CreateJavaVersionsColumnsProps {
  t: TFunction;
  isInstalling?: (version: number) => boolean;
  onInstallRecommended?: (version: number) => void;
  onDetect?: (version: number) => void;
  onBrowse?: (version: number) => void;
}

export const createJavaVersionsColumns = ({
  t,
  isInstalling,
  onInstallRecommended,
  onDetect,
  onBrowse,
}: CreateJavaVersionsColumnsProps) => {
  return [
    javaVersionsColumnHelper.accessor('majorVersion', {
      maxSize: 80,
      header: () => t('javaVersion.version'),
      cell: (cellProps) => {
        return (
          <span>
            {t('javaVersion.versionWithNumber', {
              majorVersion: cellProps.row.original.majorVersion,
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
          value={cellProps.row.original.path}
          disabled={isInstalling?.(cellProps.row.original.majorVersion)}
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
          isInstalling={isInstalling?.(cellProps.row.original.majorVersion)}
        />
      ),
      meta: {
        center: true,
      },
    }),

    javaVersionsColumnHelper.display({
      id: 'actions',
      maxSize: 80,
      cell: (cellProps) => {
        const majorVersion = () => cellProps.row.original.majorVersion;

        return (
          <JavaVersionActions
            isInstalling={isInstalling?.(cellProps.row.original.majorVersion)}
            onInstallRecommended={() => onInstallRecommended?.(majorVersion())}
            onDetect={() => onDetect?.(majorVersion())}
            onBrowse={() => onBrowse?.(majorVersion())}
          />
        );
      },
    }),
  ];
};
