import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { AppSidebar } from '@/widgets/app-sidebar';

import { CreateOfflineAccountDialog } from '@/widgets/create-offline-account-dialog';
import { CreateInstanceDialog } from '@/widgets/create-instance-dialog';
import type { RouteSectionProps } from '@solidjs/router';

export type MainLayoutProps = ComponentProps<'div'> & RouteSectionProps;

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['params', 'location', 'children']);

  return (
    <div id='main-layout' class='flex h-full bg-secondary-dark' {...others}>
      <AppSidebar
        createInstanceDialog={CreateInstanceDialog}
        createOfflineAccountDialog={CreateOfflineAccountDialog}
      />
      <div class='size-full overflow-hidden rounded-tl-2xl bg-background'>
        {local.children}
      </div>
    </div>
  );
};
