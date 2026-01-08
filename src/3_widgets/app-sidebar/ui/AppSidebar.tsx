import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import type { SidebarProps } from '@/shared/ui';

import { AccountsMenu, useAccounts } from '@/entities/accounts';
import { cn } from '@/shared/lib';
import { Separator, Sidebar } from '@/shared/ui';

import type { AccountSelectButtonProps } from './AccountSelectButton';
import type { CreateInstanceButtonProps } from './CreateInstanceButton';

import { AccountSelectButton } from './AccountSelectButton';
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
      class={cn('justify-between pt-0 pl-4 pr-1 pb-4', local.class)}
      {...others}
    >
      <div class='flex flex-col items-center gap-2'>
        <HomeButton />
        {/* <ContentButton /> */}
        <Separator />
        <CreateInstanceButton
          createInstanceDialog={local.createInstanceDialog}
        />
      </div>
      <div class='flex flex-col items-center gap-2'>
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
