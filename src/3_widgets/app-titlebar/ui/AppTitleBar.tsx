import type { Component } from 'solid-js';

import type { TitleBarProps } from '@/shared/ui';
import { TitleBar } from '@/shared/ui';

import { WindowControls } from './WindowControls';
import { ProgressMenuButton, ProgressPopover } from './ProgressMenu';

export type AppTitleBarProps = TitleBarProps;

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  return (
    <>
      <div class='fixed inset-x-0 top-0 h-[40px] w-full' />
      <TitleBar
        id='title-bar'
        class='pointer-events-auto fixed inset-x-0 top-0 z-[100] justify-end gap-2 bg-transparent'
        data-ignore-outside-click
        data-tauri-drag-region
        {...props}
      >
        <ProgressMenuButton popoverComponent={ProgressPopover} />
        <WindowControls />
      </TitleBar>
    </>
  );
};
