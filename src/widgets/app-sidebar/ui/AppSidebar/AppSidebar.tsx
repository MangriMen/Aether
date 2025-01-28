import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Separator, Sidebar } from '@/shared/ui';

import { AccountSelectButton } from '@/features/account-select-button';
import { AccountSelectCard } from '@/features/account-select-card';
import { CreateInstanceButton } from '@/features/create-instance-button';
import { HomeButton } from '@/features/home-button/';
import { SettingsButton } from '@/features/settings-button';

import type { AppSidebarProps } from '.';

export const AppSidebar: Component<AppSidebarProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <Sidebar
      class={cn('justify-between min-w-16 max-w-16', local.class)}
      {...others}
    >
      <div class='flex flex-col items-center gap-2'>
        <HomeButton />
        <Separator />
        <CreateInstanceButton />
      </div>
      <div class='flex flex-col items-center gap-2'>
        <AccountSelectButton accountSelectCard={AccountSelectCard} />
        <SettingsButton />
      </div>
    </Sidebar>
  );
};
