import { createMemo, Show, type Component } from 'solid-js';

import type { TitleBarProps } from '@/shared/ui';

import {
  UpdateNotificationStyle,
  updateNotificationStyle,
} from '@/shared/model';

import { BackForwardButtons } from './BackForwardButtons';
import { BaseTitleBar } from './BaseTitleBar';
import { ProgressMenuButton, ProgressPopover } from './ProgressMenu';
import { UpdateBanner } from './UpdateBanner';

export type AppTitleBarProps = TitleBarProps;

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  const isUpdateBannerEnabled = createMemo(
    () => updateNotificationStyle() === UpdateNotificationStyle.Banner,
  );

  return (
    <BaseTitleBar {...props}>
      <div class='min-w-39 pl-2'>
        <div class='flex max-w-15 min-w-15 justify-center'>
          <BackForwardButtons />
        </div>
      </div>
      <div class='pointer-events-none flex grow justify-center'>
        <Show when={isUpdateBannerEnabled()}>
          <UpdateBanner class='pointer-events-auto' />
        </Show>
      </div>
      <ProgressMenuButton
        class='self-end'
        popoverComponent={ProgressPopover}
        data-ignore-outside-click
      />
    </BaseTitleBar>
  );
};
