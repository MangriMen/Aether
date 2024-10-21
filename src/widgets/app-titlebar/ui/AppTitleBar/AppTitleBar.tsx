import { Icon } from '@iconify-icon/solid';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Component, createMemo, createSignal } from 'solid-js';

import { TitleBar, TitleBarButton } from '@/shared/ui';

import { NotificationMenuButton } from '@/features/notification-menu-button';

import { AppTitleBarProps } from './types';

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  const appWindow = getCurrentWebviewWindow();

  const [isMaximized, setIsMaximized] = createSignal(false);

  const handleMinimize = () => {
    appWindow.minimize();
  };

  const handleMaximize = async () => {
    const isMaximized = await appWindow.isMaximized();
    isMaximized ? appWindow.unmaximize() : appWindow.maximize();

    setIsMaximized(isMaximized);
  };

  const maximizeIcon = createMemo(() =>
    isMaximized() ? 'mdi-square-rounded-outline' : 'mdi-square-rounded',
  );

  const handleClose = () => {
    appWindow.close();
  };

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
        await handleMaximize();
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
      <div class='flex'>
        <TitleBarButton
          class='aspect-square h-full min-w-max'
          onClick={handleMinimize}
        >
          <Icon class='text-base text-muted-foreground' icon='mdi-minimize' />
        </TitleBarButton>
        <TitleBarButton
          class='aspect-square h-full min-w-max'
          onClick={handleMaximize}
        >
          <Icon class='text-base text-muted-foreground' icon={maximizeIcon()} />
        </TitleBarButton>
        <TitleBarButton
          class='aspect-square h-full min-w-max brightness-110 hover:bg-destructive'
          onClick={handleClose}
        >
          <Icon class='text-base text-muted-foreground' icon='mdi-close' />
        </TitleBarButton>
      </div>
    </TitleBar>
  );
};
