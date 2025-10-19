import type { Component, ComponentProps } from 'solid-js';

import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import IconMdiClose from '~icons/mdi/close';
import IconMdiMinimize from '~icons/mdi/minus';
import IconMdiSquareRounded from '~icons/mdi/square';
import IconMdiSquareRoundedOutline from '~icons/mdi/square-rounded-outline';
import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { isMaximized, toggleMaximize } from '@/shared/model';
import { TitleBarButton } from '@/shared/ui';

export type WindowControlsProps = ComponentProps<'div'>;

const iconClass = 'text-base text-muted-foreground';

export const WindowControls: Component<WindowControlsProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const appWindow = getCurrentWebviewWindow();

  const handleMinimize = () => {
    appWindow.minimize();
  };

  const handleClose = () => {
    appWindow.close();
  };

  return (
    <div class={cn('flex w-[120px] h-full', local.class)} {...others}>
      <TitleBarButton
        class='aspect-square h-full min-w-max'
        title='Minimize'
        onClick={handleMinimize}
      >
        <IconMdiMinimize class={iconClass} />
      </TitleBarButton>
      <TitleBarButton
        class='aspect-square h-full min-w-max'
        title='Maximize'
        onClick={toggleMaximize}
      >
        <Show
          when={isMaximized()}
          fallback={<IconMdiSquareRoundedOutline class={iconClass} />}
        >
          <IconMdiSquareRounded class={iconClass} />
        </Show>
      </TitleBarButton>
      <TitleBarButton
        class='aspect-square h-full min-w-max brightness-110 hover:bg-destructive'
        title='Close'
        onClick={handleClose}
      >
        <IconMdiClose class={iconClass} />
      </TitleBarButton>
    </div>
  );
};
