import type { ParentProps } from 'solid-js';

import { ErrorBoundary, type Component } from 'solid-js';

import { setFallbackAttributes } from '@/features/appearance';
import { isLauncherError } from '@/features/errors';
import { showWindow } from '@/platform/window';
import { logError } from '@/shared/log';

import { BaseTitleBar } from '../titlebar/BaseTitleBar';
import { AppInitializeError } from './AppInitializeError';
import { AppLayout } from './AppLayout';

export type RootErrorBoundaryProps = ParentProps;

export const RootErrorBoundary: Component<RootErrorBoundaryProps> = (props) => {
  let timer: ReturnType<typeof setTimeout> | undefined;

  const showWindowFallback = () => {
    setFallbackAttributes();
    timer = setTimeout(() => {
      showWindow().catch(() => {});
    }, 2000);
    return () => clearTimeout(timer);
  };

  return (
    <ErrorBoundary
      fallback={(err) => {
        showWindowFallback();

        logError(err);

        const error = () => {
          if (err instanceof Error) {
            return err.message;
          } else if (isLauncherError(err)) {
            return JSON.stringify(err, null, 2);
          } else if (typeof err === 'string') {
            return err;
          }

          return JSON.stringify(err, null, 2);
        };

        return (
          <AppLayout titleBar={BaseTitleBar}>
            <AppInitializeError error={error()} />
          </AppLayout>
        );
      }}
    >
      {props.children}
    </ErrorBoundary>
  );
};
