import { Icon } from '@iconify-icon/solid';
import { appWindow } from '@tauri-apps/api/window';
import { Component } from 'solid-js';

import { TitleBar, TitleBarButton } from '@/shared/ui';

import { AppTitleBarProps } from './types';
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
