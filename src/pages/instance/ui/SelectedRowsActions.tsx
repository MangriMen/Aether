import { CombinedTooltip, IconButton } from '@/shared/ui';
import type { Component, ComponentProps } from 'solid-js';
import MdiDeleteIcon from '@iconify/icons-mdi/delete';
import MdiCancelIcon from '@iconify/icons-mdi/cancel';

export type SelectedRowsActionsProps = ComponentProps<'div'>;

export const SelectedRowsActions: Component<SelectedRowsActionsProps> = (
  props,
) => {
  return (
    <div class='flex items-center gap-2' {...props}>
      <CombinedTooltip
        label='Disable'
        as={IconButton}
        class='size-8 p-0 text-lg'
        variant='ghost'
        size='sm'
        icon={MdiCancelIcon}
      />
      <CombinedTooltip
        label='Delete'
        as={IconButton}
        class='size-8 bg-destructive p-0 text-lg'
        variant='ghost'
        size='sm'
        icon={MdiDeleteIcon}
      />
    </div>
  );
};
