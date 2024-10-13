import { Icon } from '@iconify-icon/solid';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Component } from 'solid-js';

import { TitleBar, TitleBarButton } from '@/shared/ui';

import { AppTitleBarProps } from './types';
const appWindow = getCurrentWebviewWindow()
export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  return (
    <TitleBar class='rounded-lg' data-tauri-drag-region {...props}>
      <TitleBarButton onClick={() => appWindow.minimize()}>
        <Icon icon='mdi-minimize' />
      </TitleBarButton>
      <TitleBarButton onClick={() => appWindow.toggleMaximize()}>
        <Icon icon='mdi-maximize' />
      </TitleBarButton>
      <TitleBarButton onClick={() => appWindow.close()}>
        <Icon icon='mdi-close' />
      </TitleBarButton>
    </TitleBar>
  );
};
