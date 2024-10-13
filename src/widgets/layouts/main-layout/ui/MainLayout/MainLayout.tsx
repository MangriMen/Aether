import { Component, splitProps } from 'solid-js';

// eslint-disable-next-line boundaries/element-types
import { AppSidebar } from '@/widgets/app-sidebar';

import { MainLayoutProps } from '.';

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['params', 'location', 'children']);

  return (
    <div id='main-layout' class='flex h-full' {...others}>
      <AppSidebar />
      {local.children}
    </div>
  );
};
