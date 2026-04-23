import { useLocation } from '@solidjs/router';
import { createMemo, Show, type Component } from 'solid-js';

import type { TitleBarProps } from '@/shared/ui';

import { ROUTES } from '@/shared/config';
import {
  UpdateNotificationStyle,
  updateNotificationStyle,
} from '@/shared/model';

import { BaseTitleBar } from './BaseTitleBar';
import { GoHomeFromPlaygroundButton } from './GoHomeFromPlaygroundButton';
import { ProgressMenuButton, ProgressPopover } from './ProgressMenu';
import { UpdateBanner } from './UpdateBanner';

export type AppTitleBarProps = TitleBarProps;

export const AppTitleBar: Component<AppTitleBarProps> = (props) => {
  const isUpdateBannerEnabled = createMemo(
    () => updateNotificationStyle() === UpdateNotificationStyle.Banner,
  );

  const location = useLocation();

  return (
    <BaseTitleBar {...props}>
      <div class='min-w-[156px]'>
        <Show when={location.pathname === ROUTES.PLAYGROUND}>
          <GoHomeFromPlaygroundButton />
        </Show>
      </div>
      <div class='pointer-events-none flex grow justify-center'>
        <Show when={isUpdateBannerEnabled()}>
          <UpdateBanner class='pointer-events-auto' />
        </Show>
      </div>
      <ProgressMenuButton class='self-end' popoverComponent={ProgressPopover} />
    </BaseTitleBar>
  );
};
