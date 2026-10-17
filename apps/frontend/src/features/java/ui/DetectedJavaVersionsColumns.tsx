import { createColumnHelper } from '@tanstack/solid-table';
import IconMdiCheck from '~icons/mdi/check';
import { createMemo, Show, type Accessor } from 'solid-js';

import type { TFunction } from '@/shared/i18n';
import type { appTableFeatures } from '@/shared/ui-kit';

import { Button, CombinedTooltip } from '@/shared/ui-kit';

import type { StrictJavaVersion } from '../model/javaVersion';

const strictJavaVersionsColumnHelper = createColumnHelper<
  typeof appTableFeatures,
  StrictJavaVersion
>();

interface CreateDetectedJavaVersionsColumnsProps {
  t: TFunction;
  selectedVersionPath?: Accessor<string | undefined>;
  onSelect?: (javaVersion: StrictJavaVersion) => void;
}

export const createDetectedJavaVersionsColumns = ({
  t,
  selectedVersionPath,
  onSelect,
}: CreateDetectedJavaVersionsColumnsProps) => {
  const columns = createMemo(() =>
    strictJavaVersionsColumnHelper.columns([
      strictJavaVersionsColumnHelper.accessor('version', {
        header: () => t('javaVersion.version'),
      }),

      strictJavaVersionsColumnHelper.accessor('path', {
        header: () => t('javaVersion.path'),
        cell: (cellProps) => (
          <CombinedTooltip
            label={cellProps.getValue()}
            as='div'
            class='truncate'
          >
            {cellProps.renderValue()}
          </CombinedTooltip>
        ),
        meta: {
          stretch: true,
        },
      }),

      strictJavaVersionsColumnHelper.display({
        id: 'actions',
        minSize: 120,
        cell: (cellProps) => {
          const isSelected = () =>
            selectedVersionPath?.() === cellProps.row.original.path;

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
    ]),
  );

  return columns;
};
