import MdiPlusIcon from '@iconify/icons-mdi/plus';
import type { Component } from 'solid-js';
import { createSignal, lazy } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';
import { CombinedTooltip, IconButton } from '@/shared/ui';

// TODO: make something like react-modal-global

const CreateInstanceDialog = lazy(() =>
  import('@/widgets/create-instance-dialog').then((module) => ({
    default: module.CreateInstanceDialog,
  })),
);


import { useTranslate } from '@/app/model';

export type CreateInstanceButtonProps = IconButtonProps;

const CreateInstanceButton: Component<CreateInstanceButtonProps> = (props) => {
  const [{ t }] = useTranslate();

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
        {...props}
      />

      <CreateInstanceDialog
        open={isCreateInstanceDialogOpen()}
        onOpenChange={setIsCreateInstanceDialogOpen}
      />
    </>
  );
};

export default CreateInstanceButton;
