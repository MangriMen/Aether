import type { Component } from 'solid-js';

import IconMdiClose from '~icons/mdi/close';
import IconMdiPencil from '~icons/mdi/pencil';

import { CombinedTooltip, IconButton } from '@/shared/ui';

export type AllowedItemActionButtonsProps = {
  onEdit?: () => void;
  onRemove?: () => void;
};

export const AllowedItemActionButtons: Component<
  AllowedItemActionButtonsProps
> = (props) => {
  return (
    <>
      <CombinedTooltip
        label='Edit'
        as={IconButton}
        class='size-max p-1'
        variant='ghost'
        size='sm'
        icon={IconMdiPencil}
        onClick={props.onEdit}
      />
      <CombinedTooltip
        label='Remove'
        as={IconButton}
        class='size-max p-1'
        variant='ghost'
        size='sm'
        icon={IconMdiClose}
        onClick={props.onRemove}
      />
    </>
  );
};
