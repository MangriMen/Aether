import { Component, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { AddIcon, IconButton, SettingsIcon, Sidebar } from '@/shared/ui';

// TODO: make something like react-modal-global
// eslint-disable-next-line boundaries/element-types
import { CreateInstanceDialog } from '@/widgets/create-instance-dialog';

import { AppSidebarProps } from '.';

export const AppSidebar: Component<AppSidebarProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [isCreateInstanceDialogOpen, setIsCreateInstanceDialogOpen] =
    createSignal(false);

  return (
    <Sidebar class={cn('justify-end', local.class)} {...others}>
      <IconButton variant='ghost'>
        <AddIcon onClick={() => setIsCreateInstanceDialogOpen(true)} />
      </IconButton>
      <IconButton disabled variant='ghost'>
        <SettingsIcon />
      </IconButton>

      {/* Dialogs */}
      <CreateInstanceDialog
        open={isCreateInstanceDialogOpen()}
        onOpenChange={setIsCreateInstanceDialogOpen}
      />
    </Sidebar>
  );
};
