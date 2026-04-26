import { ErrorBoundary, type Component, type JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import type { TitleBarProps } from '../../shared/ui';

import { logError } from '../../shared/lib';
import { AppErrorBoundary } from './AppErrorBoundary';

export type AppLayoutProps = {
  children?: JSX.Element;
  titleBar: Component<TitleBarProps>;
};

export const AppLayout: Component<AppLayoutProps> = (props) => {
  return (
    <>
      <Dynamic component={props.titleBar} />
      <div class='mt-titlebar flex size-full flex-col overflow-hidden'>
        <ErrorBoundary
          fallback={(err, reset) => {
            logError(err);
            return <AppErrorBoundary reset={reset} />;
          }}
        >
          {props.children}
        </ErrorBoundary>
      </div>
    </>
  );
};
