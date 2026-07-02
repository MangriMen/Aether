import { splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/lib';
import { TitleBar, type TitleBarProps } from '@/shared/ui';

import { WindowControls } from './WindowControls';

export type BaseTitleBarProps = TitleBarProps;

export const BaseTitleBar: Component<BaseTitleBarProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'children']);

  return (
    <>
      <div class='inset-x-0 top-0 h-titlebar fixed w-full' />
      <TitleBar
        id='title-bar'
        class={cn(
          `
            inset-x-0 top-0 gap-2 pointer-events-auto fixed z-100 items-center
            bg-transparent
          `,
          local.class,
        )}
        data-ignore-outside-click
        data-tauri-drag-region
        {...others}
      >
        {local.children}
        <WindowControls
          class='w-30.25 min-w-30.25 ml-auto self-end pt-px pr-px'
          data-ignore-outside-click
        />
      </TitleBar>
    </>
  );
};
