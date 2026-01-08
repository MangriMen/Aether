import { createMemo, Show, type Component } from 'solid-js';

import type { TitleBarProps } from '@/shared/ui';

import {
  UpdateNotificationStyle,
  updateNotificationStyle,
} from '@/shared/model';
import { TitleBar } from '@/shared/ui';

import { ProgressMenuButton, ProgressPopover } from './ProgressMenu';
import { UpdateBanner } from './UpdateBanner';
import { WindowControls } from './WindowControls';

export type AppTitleBarProps = TitleBarProps;

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  const isUpdateBannerEnabled = createMemo(
    () => updateNotificationStyle() === UpdateNotificationStyle.Banner,
  );

  return (
    <>
      <div class='h-titlebar fixed inset-x-0 top-0 w-full' />
      <TitleBar
        id='title-bar'
        class='pointer-events-auto fixed inset-x-0 top-0 z-[100] items-center gap-2 bg-transparent'
        data-ignore-outside-click
        data-tauri-drag-region
        {...props}
      >
        <div class='min-w-[156px]' />
        <div class='pointer-events-none flex grow justify-center'>
          <Show when={isUpdateBannerEnabled()}>
            <UpdateBanner class='pointer-events-auto' />
          </Show>
        </div>
        <ProgressMenuButton
          class='self-end'
          popoverComponent={ProgressPopover}
        />
        <WindowControls class='w-[121px] min-w-[121px] self-end pr-px pt-px' />
      </TitleBar>
    </>
  );
};
