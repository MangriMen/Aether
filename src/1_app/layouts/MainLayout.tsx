import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { AppSidebar } from '@/widgets/app-sidebar';
import { CreateInstanceDialog } from '@/widgets/create-instance-dialog';
import { CreateOfflineAccountDialog } from '@/widgets/create-offline-account-dialog';

export type MainLayoutProps = ComponentProps<'div'>;

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['children']);

  return (
    <div class='flex h-full' id='main-layout' {...others}>
      <AppSidebar
        createInstanceDialog={CreateInstanceDialog}
        createOfflineAccountDialog={CreateOfflineAccountDialog}
      />
      <div class='size-full overflow-hidden'>{local.children}</div>
    </div>
  );
};
