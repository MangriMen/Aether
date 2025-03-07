import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import type { SidebarProps } from '@/shared/ui';
import { Separator, Sidebar } from '@/shared/ui';

import type { AccountSelectButtonProps } from './AccountSelectButton';
import { AccountSelectButton } from './AccountSelectButton';
import type { CreateInstanceButtonProps } from './CreateInstanceButton';
import CreateInstanceButton from './CreateInstanceButton';
import HomeButton from './HomeButton';
import SettingsButton from './SettingsButton';
import { AccountsMenu } from '@/entities/accounts';

export type AppSidebarProps = SidebarProps &
  Pick<CreateInstanceButtonProps, 'createInstanceDialog'> &
  Pick<AccountSelectButtonProps, 'createOfflineAccountDialog'>;

export const AppSidebar: Component<AppSidebarProps> = (props) => {
  const [local, others] = splitProps(props, [
    'createInstanceDialog',
    'createOfflineAccountDialog',
    'class',
  ]);

  return (
    <Sidebar
      class={cn('justify-between min-w-16 max-w-16', local.class)}
      {...others}
    >
      <div class='flex flex-col items-center gap-2'>
        <HomeButton />
        <Separator />
        <CreateInstanceButton
          createInstanceDialog={local.createInstanceDialog}
        />
      </div>
      <div class='flex flex-col items-center gap-2'>
        <AccountSelectButton
          accountSelectCard={AccountsMenu}
          createOfflineAccountDialog={local.createOfflineAccountDialog}
        />
        <SettingsButton />
      </div>
    </Sidebar>
  );
};
