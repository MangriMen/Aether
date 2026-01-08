import { createMemo, Show, type Component } from 'solid-js';

import type { TitleBarProps } from '@/shared/ui';

import {
  UpdateNotificationStyle,
  updateNotificationStyle,
} from '@/shared/model';
import { TitleBar } from '@/shared/ui';

import { ProgressMenuButton, ProgressPopover } from './ProgressMenu';
import { UpdateBadge } from './UpdateBadge';
import { WindowControls } from './WindowControls';

export type AppTitleBarProps = TitleBarProps;

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  const isUpdateBannerEnabled = createMemo(
    () => updateNotificationStyle() === UpdateNotificationStyle.Banner,
  );

  return (
    <>
      <div class='fixed inset-x-0 top-0 h-[30px] w-full' />
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
            <UpdateBadge class='pointer-events-auto' />
          </Show>
        </div>
        <ProgressMenuButton
          class='self-end'
          popoverComponent={ProgressPopover}
        />
        <WindowControls class='w-[120px] min-w-[120px] self-end' />
      </TitleBar>
    </>
  );
};
