import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';


import { AppSidebar } from '@/widgets/app-sidebar';

import type { MainLayoutProps } from '.';

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['params', 'location', 'children']);

  return (
    <div id='main-layout' class='flex h-full bg-secondary-dark' {...others}>
      <AppSidebar />
      <div class='size-full overflow-hidden rounded-tl-2xl bg-background'>
        {local.children}
      </div>
    </div>
  );
};
