import { Component } from 'solid-js';

import { TitleBar } from '@/shared/ui';

import { NotificationMenuButton } from '@/features/notification-menu-button';
import { WindowControls } from '@/features/window-controls';

import { AppTitleBarProps } from './types';

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  return (
    <>
      <div class='fixed inset-x-0 top-0 h-[40px] w-full bg-secondary-dark' />
      <TitleBar
        id='title-bar'
        class='pointer-events-auto fixed inset-x-0 top-0 z-[100] justify-end gap-2 bg-transparent'
        data-ignore-outside-click
        data-tauri-drag-region
        {...props}
      >
        <NotificationMenuButton />
        <WindowControls />
      </TitleBar>
    </>
  );
};
