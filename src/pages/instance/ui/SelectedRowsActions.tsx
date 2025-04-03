import { CombinedTooltip, IconButton } from '@/shared/ui';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import MdiDeleteIcon from '@iconify/icons-mdi/delete';
import MdiCancelIcon from '@iconify/icons-mdi/cancel';
import MdiCheckIcon from '@iconify/icons-mdi/check';
import {
  type Instance,
  enableInstanceContents,
  type InstanceFile,
  disableInstanceContents,
  removeInstanceContents,
} from '@/entities/instances';

import type { RowModel } from '@tanstack/solid-table';
import { useTranslate } from '@/shared/model';
import { cn } from '@/shared/lib';

export type SelectedRowsActionsProps = ComponentProps<'div'> & {
  instanceId: Instance['id'];
  selectedRows: RowModel<InstanceFile>;
};

export const SelectedRowsActions: Component<SelectedRowsActionsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'selectedRows',
    'class',
  ]);

  const [{ t }] = useTranslate();

  const selectedContents = createMemo(() =>
    local.selectedRows.rows.map((row) => row.original),
  );

  const selectionDisableStatus = createMemo(() =>
    selectedContents().reduce(
      (acc, item) => {
        acc[item.disabled ? 'disabled' : 'enabled']++;
        return acc;
      },
      { disabled: 0, enabled: 0 },
    ),
  );

  const getContentPaths = () => {
    return selectedContents().map((content) => content.path);
  };

  const handleEnableContents = () => {
    enableInstanceContents(local.instanceId, getContentPaths());
  };

  const handleDisableContents = () => {
    disableInstanceContents(local.instanceId, getContentPaths());
  };

  const handleDeleteContents = () => {
    removeInstanceContents(local.instanceId, getContentPaths());
  };

  return (
    <div
      class={cn('flex items-center gap-2 text-foreground', local.class)}
      {...others}
    >
      <Show when={selectionDisableStatus().enabled > 0}>
        <CombinedTooltip
          label={t('common.disable')}
          as={IconButton}
          class='size-8 p-0 text-lg'
          variant='ghost'
          size='sm'
          icon={MdiCancelIcon}
          onClick={handleDisableContents}
        />
      </Show>
      <Show when={selectionDisableStatus().disabled > 0}>
        <CombinedTooltip
          label={t('common.enable')}
          as={IconButton}
          class='size-8 p-0 text-lg'
          variant='ghost'
          size='sm'
          icon={MdiCheckIcon}
          onClick={handleEnableContents}
        />
      </Show>
      <CombinedTooltip
        label={t('common.delete')}
        as={IconButton}
        class='size-8 p-0 text-lg'
        variant='destructive'
        size='sm'
        icon={MdiDeleteIcon}
        onClick={handleDeleteContents}
      />
    </div>
  );
};
