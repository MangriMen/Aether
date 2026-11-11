import type { RowModel } from '@tanstack/solid-table';
import type { Component, ComponentProps } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import type { ContentFile } from '@/features/content';
import type { Instance } from '@/features/instance';
import type { AppTableFeatures } from '@/shared/ui-kit';

import { useTranslation } from '@/shared/i18n';
import { Button } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { SelectedRowsActions } from './SelectedRowsActions';

export type SelectionToolbarProps = ComponentProps<'div'> & {
  instanceId: Instance['id'];
  selectedRows: RowModel<AppTableFeatures, ContentFile>;
  /** Rows that pass the filter, on every page. */
  totalCount: number;
  onSelectAll: () => void;
  onClear: () => void;
};

/** Takes the place of the table header's column cells while rows are selected. */
export const SelectionToolbar: Component<SelectionToolbarProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'selectedRows',
    'totalCount',
    'onSelectAll',
    'onClear',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const selectedCount = () => local.selectedRows.rows.length;

  return (
    // `contain` keeps the toolbar's own width out of the table's column sizing.
    <div
      class={cn(
        'w-full min-w-0 gap-1 text-sm font-normal contain-[inline-size] flex items-center',
        local.class,
      )}
      {...others}
    >
      <span class='mr-1 truncate text-foreground'>
        {t('instance.selectedContents', {
          selected: selectedCount(),
          total: local.totalCount,
        })}
      </span>
      <Show when={selectedCount() < local.totalCount}>
        <Button variant='ghost' size='sm' onClick={local.onSelectAll}>
          {t('instance.selectAllContents', { total: local.totalCount })}
        </Button>
      </Show>
      <Button variant='ghost' size='sm' onClick={local.onClear}>
        {t('instance.clearSelection')}
      </Button>
      <SelectedRowsActions
        class='ml-auto'
        instanceId={local.instanceId}
        selectedRows={local.selectedRows}
        onRemoved={local.onClear}
      />
    </div>
  );
};
