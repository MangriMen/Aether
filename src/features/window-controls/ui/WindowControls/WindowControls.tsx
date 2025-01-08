import { Icon } from '@iconify-icon/solid';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Component, createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { isMaximized, toggleMaximize } from '@/shared/model';
import { TitleBarButton } from '@/shared/ui';

import { WindowControlsProps } from './types';

export const WindowControls: Component<WindowControlsProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const appWindow = getCurrentWebviewWindow();

  const handleMinimize = () => {
    appWindow.minimize();
  };

  const maximizeIcon = createMemo(() =>
    isMaximized() ? 'mdi-square-rounded' : 'mdi-square-rounded-outline',
  );

  const handleClose = () => {
    appWindow.close();
  };

  return (
    <div class={cn('flex', local.class)} {...others}>
      <TitleBarButton
        class='aspect-square h-full min-w-max'
        title='Minimize'
        onClick={handleMinimize}
      >
        <Icon class='text-base text-muted-foreground' icon='mdi-minimize' />
      </TitleBarButton>
      <TitleBarButton
        class='aspect-square h-full min-w-max'
        title='Maximize'
        onClick={toggleMaximize}
      >
        <Icon class='text-base text-muted-foreground' icon={maximizeIcon()} />
      </TitleBarButton>
      <TitleBarButton
        class='aspect-square h-full min-w-max brightness-110 hover:bg-destructive'
        title='Close'
        onClick={handleClose}
      >
        <Icon class='text-base text-muted-foreground' icon='mdi-close' />
      </TitleBarButton>
    </div>
  );
};
