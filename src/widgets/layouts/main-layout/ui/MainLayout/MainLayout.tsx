import { Component, splitProps } from 'solid-js';

// eslint-disable-next-line boundaries/element-types
import { AppSidebar } from '@/widgets/app-sidebar';

import { MainLayoutProps } from '.';

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['params', 'location', 'children']);

  return (
    <div id='main-layout' class='flex h-full bg-secondary-dark' {...others}>
      <AppSidebar />
      <div class='size-full overflow-hidden rounded-l-2xl bg-background'>
        {local.children}
      </div>
    </div>
  );
};
