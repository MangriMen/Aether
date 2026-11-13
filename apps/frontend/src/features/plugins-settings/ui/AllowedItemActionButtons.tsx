import type { Component } from 'solid-js';

import IconMdiClose from '~icons/mdi/close';
import IconMdiPencil from '~icons/mdi/pencil';

import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';

export type AllowedItemActionButtonsProps = {
  onEdit?: () => void;
  onRemove?: () => void;
};

export const AllowedItemActionButtons: Component<
  AllowedItemActionButtonsProps
> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <>
      <CombinedTooltip
        label={t('common.edit')}
        as={IconButton}
        class='p-1 size-max'
        variant='ghost'
        size='sm'
        icon={IconMdiPencil}
        onClick={props.onEdit}
      />
      <CombinedTooltip
        label={t('common.remove')}
        as={IconButton}
        class='p-1 size-max'
        variant='ghost'
        size='sm'
        icon={IconMdiClose}
        onClick={props.onRemove}
      />
    </>
  );
};
