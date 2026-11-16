import type { Component, ComponentProps } from 'solid-js';

import { useLocation, useNavigate } from '@solidjs/router';
import { createEffect, ErrorBoundary, on, splitProps } from 'solid-js';

import { CreateOfflineAccountDialog } from '@/features/account';
import { CreateInstanceDialog } from '@/features/instance-create';
import { logError } from '@/shared/log';

import { APP_ROUTES } from '../appRoutes';
import { AppSidebar } from '../sidebar/AppSidebar';
import { AppErrorBoundary } from './AppErrorBoundary';

export type MainLayoutProps = ComponentProps<'div'>;

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [local, others] = splitProps(props, ['children']);

  const location = useLocation();
  const navigate = useNavigate();

  let resetBoundary: (() => void) | undefined;

  // Resetting in place would render the broken route again, so go home first;
  // any pathname change clears the boundary.
  createEffect(
    on(
      () => location.pathname,
      () => resetBoundary?.(),
      { defer: true },
    ),
  );

  const handleReset = () => {
    if (location.pathname === APP_ROUTES.HOME) {
      resetBoundary?.();
    } else {
      navigate(APP_ROUTES.HOME);
    }
  };

  return (
    <div id='main-layout' class='flex h-full' {...others}>
      <AppSidebar
        createInstanceDialog={CreateInstanceDialog}
        createOfflineAccountDialog={CreateOfflineAccountDialog}
      />
      <div class='size-full overflow-hidden'>
        {/* A render error in a page keeps the sidebar; leaving the route clears it. */}
        <ErrorBoundary
          fallback={(err, reset) => {
            logError(err);
            resetBoundary = reset;
            return <AppErrorBoundary reset={handleReset} />;
          }}
        >
          {local.children}
        </ErrorBoundary>
      </div>
    </div>
  );
};
