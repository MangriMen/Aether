import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps } from 'solid-js';

import MdiPlusIcon from '@iconify/icons-mdi/plus';
import { createSignal, splitProps } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type CreateInstanceButtonProps = {
  createInstanceDialog: Component<ComponentProps<'div'> & DialogRootProps>;
} & IconButtonProps;

const CreateInstanceButton: Component<CreateInstanceButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['createInstanceDialog']);
  const [{ t }] = useTranslation();

  const [isCreateInstanceDialogOpen, setIsCreateInstanceDialogOpen] =
    createSignal(false);

  const handleClick = () => setIsCreateInstanceDialogOpen(true);

  return (
    <>
      <CombinedTooltip
        as={IconButton}
        icon={MdiPlusIcon}
        label={t('common.createInstance')}
        onClick={handleClick}
        placement='right'
        {...others}
      />

      <local.createInstanceDialog
        onOpenChange={setIsCreateInstanceDialogOpen}
        open={isCreateInstanceDialogOpen()}
      />
    </>
  );
};

export default CreateInstanceButton;
