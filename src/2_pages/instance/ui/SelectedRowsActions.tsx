import type { RowModel } from '@tanstack/solid-table';

import MdiCancelIcon from '@iconify/icons-mdi/cancel';
import MdiCheckIcon from '@iconify/icons-mdi/check';
import MdiDeleteIcon from '@iconify/icons-mdi/delete';
import {
  type Component,
  type ComponentProps,
  createMemo,
  Show,
  splitProps,
} from 'solid-js';

import {
  type Instance,
  type InstanceFile,
  useDisableContents,
  useEnableContents,
  useRemoveContents,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type SelectedRowsActionsProps = {
  instanceId: Instance['id'];
  selectedRows: RowModel<InstanceFile>;
} & ComponentProps<'div'>;

export const SelectedRowsActions: Component<SelectedRowsActionsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'selectedRows',
    'class',
  ]);

  const [{ t }] = useTranslation();

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

  const { mutateAsync: enableInstanceContents } = useEnableContents();
  const { mutateAsync: disableInstanceContents } = useDisableContents();
  const { mutateAsync: removeInstanceContents } = useRemoveContents();

  const handleEnableContents = () => {
    enableInstanceContents({ id: local.instanceId, paths: getContentPaths() });
  };

  const handleDisableContents = () => {
    disableInstanceContents({ id: local.instanceId, paths: getContentPaths() });
  };

  const handleDeleteContents = () => {
    removeInstanceContents({ id: local.instanceId, paths: getContentPaths() });
  };

  return (
    <div
      class={cn('flex items-center gap-2 text-foreground', local.class)}
      {...others}
    >
      <Show when={selectionDisableStatus().enabled > 0}>
        <CombinedTooltip
          as={IconButton}
          class='size-8 p-0 text-lg'
          icon={MdiCancelIcon}
          label={t('common.disable')}
          onClick={handleDisableContents}
          size='sm'
          variant='ghost'
        />
      </Show>
      <Show when={selectionDisableStatus().disabled > 0}>
        <CombinedTooltip
          as={IconButton}
          class='size-8 p-0 text-lg'
          icon={MdiCheckIcon}
          label={t('common.enable')}
          onClick={handleEnableContents}
          size='sm'
          variant='ghost'
        />
      </Show>
      <CombinedTooltip
        as={IconButton}
        class='size-8 p-0 text-lg'
        icon={MdiDeleteIcon}
        label={t('common.delete')}
        onClick={handleDeleteContents}
        size='sm'
        variant='destructive'
      />
    </div>
  );
};
