import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Component } from 'solid-js';

import { toggleMaximize } from '@/shared/model';
import { TitleBar } from '@/shared/ui';

import { NotificationMenuButton } from '@/features/notification-menu-button';
import { WindowControls } from '@/features/window-controls';

import { AppTitleBarProps } from './types';

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  const appWindow = getCurrentWebviewWindow();

  let timerId: number | null = null;

  const handleDoubleClick = async (e: MouseEvent) => {
    if (e.target !== e.currentTarget) {
      return;
    }

    if (e.buttons === 1) {
      if (e.detail === 1) {
        if (timerId !== null) {
          clearTimeout(timerId);
        }
        timerId = setTimeout(() => appWindow.startDragging(), 80);
      } else if (e.detail === 2) {
        if (timerId !== null) {
          clearTimeout(timerId);
        }
        await toggleMaximize();
      }
    }
  };

  return (
    <TitleBar
      id='title-bar'
      class='justify-end gap-2'
      // data-tauri-drag-region
      onMouseDown={handleDoubleClick}
      {...props}
    >
      <NotificationMenuButton />
      <WindowControls />
    </TitleBar>
  );
};
