import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import type { SidebarProps } from '@/shared/ui';

import { AccountsMenu, useAccounts } from '@/entities/account';
import { cn } from '@/shared/lib';
import { Separator, Sidebar } from '@/shared/ui';

import type { AccountSelectButtonProps } from './AccountSelectButton';
import type { CreateInstanceButtonProps } from './CreateInstanceButton';

import { AccountSelectButton } from './AccountSelectButton';
import { ContentButton } from './ContentButton';
import CreateInstanceButton from './CreateInstanceButton';
import HomeButton from './HomeButton';
import SettingsButton from './SettingsButton';

export type AppSidebarProps = SidebarProps &
  Pick<CreateInstanceButtonProps, 'createInstanceDialog'> &
  Pick<AccountSelectButtonProps, 'createOfflineAccountDialog'>;

export const AppSidebar: Component<AppSidebarProps> = (props) => {
  const [local, others] = splitProps(props, [
    'createInstanceDialog',
    'createOfflineAccountDialog',
    'class',
  ]);

  const accounts = useAccounts();

  return (
    <Sidebar
      class={cn('pr-1 pb-4 pl-4 justify-between pt-px', local.class)}
      {...others}
    >
      <div class='gap-2 flex flex-col items-center'>
        <HomeButton />
        <ContentButton />
        <Separator />
        <CreateInstanceButton
          createInstanceDialog={local.createInstanceDialog}
        />
      </div>
      <div class='gap-2 flex flex-col items-center'>
        <AccountSelectButton
          accounts={accounts.data ?? []}
          accountsMenu={AccountsMenu}
          createOfflineAccountDialog={local.createOfflineAccountDialog}
        />
        <SettingsButton />
      </div>
    </Sidebar>
  );
};
