import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps } from 'solid-js';

import MdiPlusIcon from '@iconify/icons-mdi/plus';
import { createSignal, splitProps } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type CreateInstanceButtonProps = IconButtonProps & {
  createInstanceDialog: Component<ComponentProps<'div'> & DialogRootProps>;
};

const CreateInstanceButton: Component<CreateInstanceButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['createInstanceDialog']);
  const [{ t }] = useTranslation();

  const [isCreateInstanceDialogOpen, setIsCreateInstanceDialogOpen] =
    createSignal(false);

  const handleClick = () => setIsCreateInstanceDialogOpen(true);

  return (
    <>
      <CombinedTooltip
        label={t('common.createInstance')}
        placement='right'
        as={IconButton}
        onClick={handleClick}
        icon={MdiPlusIcon}
        {...others}
      />

      <local.createInstanceDialog
        open={isCreateInstanceDialogOpen()}
        onOpenChange={setIsCreateInstanceDialogOpen}
      />
    </>
  );
};

export default CreateInstanceButton;
