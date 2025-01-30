import type { Component, JSX } from 'solid-js';

import { AppTitleBar } from '@/widgets/app-titlebar';

export type AppLayoutProps = {
  children?: JSX.Element;
};

export const AppLayout: Component<AppLayoutProps> = (props) => {
  return (
    <>
      <AppTitleBar />
      <div class='mt-[40px] flex size-full flex-col'>{props.children}</div>
    </>
  );
};
