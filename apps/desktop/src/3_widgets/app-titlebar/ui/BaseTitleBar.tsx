import { splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/lib';
import { TitleBar, type TitleBarProps } from '@/shared/ui';

import { WindowControls } from './WindowControls';

export type BaseTitleBarProps = TitleBarProps;

export const BaseTitleBar: Component<BaseTitleBarProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'children']);

  return (
    <>
      <div class='h-titlebar fixed inset-x-0 top-0 w-full' />
      <TitleBar
        id='title-bar'
        class={cn(
          'pointer-events-auto fixed inset-x-0 top-0 z-[100] items-center gap-2 bg-transparent',
          local.class,
        )}
        data-ignore-outside-click
        data-tauri-drag-region
        {...others}
      >
        {local.children}
        <WindowControls class='ml-auto w-[121px] min-w-[121px] self-end pr-px pt-px' />
      </TitleBar>
    </>
  );
};
