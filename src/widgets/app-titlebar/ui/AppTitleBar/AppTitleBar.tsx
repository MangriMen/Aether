import { Component } from 'solid-js';

import { TitleBar } from '@/shared/ui';

import { NotificationMenuButton } from '@/features/notification-menu-button';
import { WindowControls } from '@/features/window-controls';

import { AppTitleBarProps } from './types';

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  return (
    <TitleBar
      id='title-bar'
      class='pointer-events-auto z-[100] justify-end gap-2'
      data-ignore-outside-click
      data-tauri-drag-region
      {...props}
    >
      <NotificationMenuButton />
      <WindowControls />
    </TitleBar>
  );
};
