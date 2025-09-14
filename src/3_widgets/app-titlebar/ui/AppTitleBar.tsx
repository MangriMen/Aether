import type { Component } from 'solid-js';

import type { TitleBarProps } from '@/shared/ui';

import { TitleBar } from '@/shared/ui';

import { ProgressMenuButton, ProgressPopover } from './ProgressMenu';
import { WindowControls } from './WindowControls';

export type AppTitleBarProps = TitleBarProps;

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  return (
    <>
      <div class='fixed inset-x-0 top-0 h-[40px] w-full' />
      <TitleBar
        class='pointer-events-auto fixed inset-x-0 top-0 z-[100] items-center justify-end gap-2 bg-transparent'
        data-ignore-outside-click
        data-tauri-drag-region
        id='title-bar'
        {...props}
      >
        <ProgressMenuButton popoverComponent={ProgressPopover} />
        <WindowControls />
      </TitleBar>
    </>
  );
};
