import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { CreateOfflineAccountDialog } from '@/features/account';
import { CreateInstanceDialog } from '@/features/instance-create';

import { AppSidebar } from '../sidebar/AppSidebar';

export type MainLayoutProps = ComponentProps<'div'>;

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['children']);

  return (
    <div id='main-layout' class='flex h-full' {...others}>
      <AppSidebar
        createInstanceDialog={CreateInstanceDialog}
        createOfflineAccountDialog={CreateOfflineAccountDialog}
      />
      <div class='size-full overflow-hidden'>{local.children}</div>
    </div>
  );
};
