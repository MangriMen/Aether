import { Component, splitProps } from 'solid-js';

import { AppSidebar } from '@/widgets/app-sidebar';

import { HomePageLayoutProps } from '.';

export const HomePageLayout: Component<HomePageLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['children']);

  return (
    <div class='flex h-full' {...others}>
      <AppSidebar />
      {local.children}
    </div>
  );
};
