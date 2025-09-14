import type { Component, ComponentProps } from 'solid-js';

import { Icon } from '@iconify-icon/solid';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { isMaximized, toggleMaximize } from '@/shared/model';
import { TitleBarButton } from '@/shared/ui';

export type WindowControlsProps = ComponentProps<'div'>;

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
    <div class={cn('flex w-[120px] h-full', local.class)} {...others}>
      <TitleBarButton
        class='aspect-square h-full min-w-max'
        onClick={handleMinimize}
        title='Minimize'
      >
        <Icon class='text-base text-muted-foreground' icon='mdi-minimize' />
      </TitleBarButton>
      <TitleBarButton
        class='aspect-square h-full min-w-max'
        onClick={toggleMaximize}
        title='Maximize'
      >
        <Icon class='text-base text-muted-foreground' icon={maximizeIcon()} />
      </TitleBarButton>
      <TitleBarButton
        class='aspect-square h-full min-w-max brightness-110 hover:bg-destructive'
        onClick={handleClose}
        title='Close'
      >
        <Icon class='text-base text-muted-foreground' icon='mdi-close' />
      </TitleBarButton>
    </div>
  );
};
