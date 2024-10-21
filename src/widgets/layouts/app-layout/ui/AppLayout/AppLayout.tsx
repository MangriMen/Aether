import { Component } from 'solid-js';

// eslint-disable-next-line boundaries/element-types
import { AppTitleBar } from '@/widgets/app-titlebar';

import { AppLayoutProps } from './types';

export const AppLayout: Component<AppLayoutProps> = (props) => {
  return (
    <>
      <AppTitleBar />
      <div class='mt-[40px] flex size-full flex-col'>{props.children}</div>
    </>
  );
};
