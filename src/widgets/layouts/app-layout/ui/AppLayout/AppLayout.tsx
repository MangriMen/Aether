import type { Component } from 'solid-js';

import { AppTitleBar } from '@/widgets/app-titlebar';

import type { AppLayoutProps } from './types';

export const AppLayout: Component<AppLayoutProps> = (props) => {
  return (
    <>
      <AppTitleBar />
      <div class='mt-[40px] flex size-full flex-col'>{props.children}</div>
    </>
  );
};
