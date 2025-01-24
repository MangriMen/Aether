import { A } from '@solidjs/router';
import { Component, createSignal, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  AddIcon,
  Badge,
  HomeIcon,
  IconButton,
  SettingsIcon,
  Sidebar,
} from '@/shared/ui';

import { updateResource } from '@/entities/update';

// eslint-disable-next-line boundaries/element-types
import { AccountSelectButton } from '@/widgets/account-select-button';
// TODO: make something like react-modal-global
// eslint-disable-next-line boundaries/element-types
import { CreateInstanceDialog } from '@/widgets/create-instance-dialog';

import { AppSidebarProps } from '.';

export const AppSidebar: Component<AppSidebarProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [update] = updateResource;

  const [isCreateInstanceDialogOpen, setIsCreateInstanceDialogOpen] =
    createSignal(false);

  return (
    <Sidebar
      class={cn('justify-between min-w-16 max-w-16', local.class)}
      {...others}
    >
      <div class='flex flex-col items-center gap-2'>
        <IconButton as={A} href='/' variant='ghost' title='Home'>
          <HomeIcon />
        </IconButton>
        <hr class='h-px w-full bg-muted' />
        <IconButton
          variant='ghost'
          onClick={() => setIsCreateInstanceDialogOpen(true)}
          title='Create instance'
        >
          <AddIcon />
        </IconButton>
      </div>
      <div class='flex flex-col items-center gap-2'>
        <AccountSelectButton />
        <IconButton
          class='relative'
          as={A}
          href='/settings'
          variant='ghost'
          title={
            update()?.available ? 'Settings (Update available)' : `Settings`
          }
        >
          <Show when={update()?.available}>
            <Badge class='absolute right-2 top-1 aspect-square size-2 p-0' />
          </Show>
          <SettingsIcon />
        </IconButton>
      </div>

      {/* Dialogs */}
      <CreateInstanceDialog
        open={isCreateInstanceDialogOpen()}
        onOpenChange={setIsCreateInstanceDialogOpen}
      />
    </Sidebar>
  );
};
