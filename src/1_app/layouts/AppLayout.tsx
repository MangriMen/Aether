import { ErrorBoundary, type Component, type JSX } from 'solid-js';

import { logError } from '@/shared/lib/log';
import { AppTitleBar } from '@/widgets/app-titlebar';

import { AppErrorBoundary } from './AppErrorBoundary';

export type AppLayoutProps = {
  children?: JSX.Element;
};

export const AppLayout: Component<AppLayoutProps> = (props) => {
  return (
    <>
      <AppTitleBar />
      <div class='mt-[30px] flex size-full flex-col overflow-hidden'>
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
