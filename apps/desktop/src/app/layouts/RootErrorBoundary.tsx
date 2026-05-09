import type { ParentProps } from 'solid-js';

import { ErrorBoundary, type Component } from 'solid-js';

import { logError } from '@/shared/lib';
import { isLauncherError } from '@/shared/model';
import { BaseTitleBar } from '@/widgets/app-titlebar';

import { setFallbackAttributes } from '../lib';
import { showWindow } from '../model';
import { AppInitializeError } from '../providers';
import { AppLayout } from './AppLayout';

export type RootErrorBoundaryProps = ParentProps;

export const RootErrorBoundary: Component<RootErrorBoundaryProps> = (props) => {
  let timer = 0;

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
