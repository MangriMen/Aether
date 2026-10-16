import type { Component, ComponentProps } from 'solid-js';

import { useLocation } from '@solidjs/router';
import { Show, splitProps } from 'solid-js';

import { CreateInstanceDialog } from '@/features/instance-create';
import { ROUTES } from '@/shared/config';
import { CreateOfflineAccountDialog } from '@/widgets/create-offline-account-dialog';

import { AppSidebar } from '../sidebar/AppSidebar';

export type MainLayoutProps = ComponentProps<'div'>;

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['children']);

  const location = useLocation();

  return (
    <div id='main-layout' class='flex h-full' {...others}>
      <Show when={location.pathname !== ROUTES.PLAYGROUND}>
        <AppSidebar
          createInstanceDialog={CreateInstanceDialog}
          createOfflineAccountDialog={CreateOfflineAccountDialog}
        />
      </Show>
      <div class='size-full overflow-hidden'>{local.children}</div>
    </div>
  );
};
