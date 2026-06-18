import type { Component, ComponentProps } from 'solid-js';

import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import IconMdiClose from '~icons/mdi/close';
import IconMdiMinimize from '~icons/mdi/minus';
import IconMdiSquareRounded from '~icons/mdi/square-rounded';
import IconMdiSquareRoundedOutline from '~icons/mdi/square-rounded-outline';
import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { isMaximized, toggleMaximize } from '@/shared/model';
import { TitleBarButton } from '@/shared/ui';

export type WindowControlsProps = ComponentProps<'div'>;

const iconClass = 'pointer-events-none fill-current text-sm';

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
    <div
      class={cn('flex h-full text-muted-foreground', local.class)}
      data-ignore-outside-click
      {...others}
    >
      <TitleBarButton
        title='Minimize'
        onClick={handleMinimize}
        data-ignore-outside-click
      >
        <IconMdiMinimize class={iconClass} />
      </TitleBarButton>
      <TitleBarButton
        title='Maximize'
        onClick={toggleMaximize}
        data-ignore-outside-click
      >
        <Show
          when={isMaximized()}
          fallback={<IconMdiSquareRoundedOutline class={iconClass} />}
        >
          <IconMdiSquareRounded class={iconClass} />
        </Show>
      </TitleBarButton>
      <TitleBarButton
        class='enabled:hover:bg-destructive/75'
        title='Close'
        onClick={handleClose}
        data-ignore-outside-click
      >
        <IconMdiClose class={iconClass} />
      </TitleBarButton>
    </div>
  );
};
