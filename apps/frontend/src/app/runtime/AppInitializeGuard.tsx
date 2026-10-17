import type { Component, JSX } from 'solid-js';

import { createSignal, onMount, Show } from 'solid-js';

import { AppInitializeError, AppLayout, BaseTitleBar } from '@/app/shell';
import { getTranslatedError, isLauncherError } from '@/features/errors';
import { useTranslation } from '@/shared/i18n';

import { setupApp } from './initialize';

export type AppInitializeGuardProps = { children?: JSX.Element };

export const AppInitializeGuard: Component<AppInitializeGuardProps> = (
  props,
) => {
  const [isReady, setIsReady] = createSignal(false);
  const [error, setError] = createSignal<string | undefined>();

  const [{ t }] = useTranslation();

  const handleSetup = async () => {
    setIsReady(false);
    setError(undefined);

    try {
      await setupApp();
    } catch (err) {
      setError(() => {
        if (err instanceof Error) {
          return err.message;
        } else if (isLauncherError(err)) {
          return getTranslatedError(err, t);
        } else if (typeof err === 'string') {
          return err;
        }

        return JSON.stringify(err, null, 2);
      });
    } finally {
      setIsReady(true);
    }
  };

  onMount(() => handleSetup());

  return (
    <Show when={isReady()}>
      <Show
        when={!error()}
        fallback={
          <AppLayout titleBar={BaseTitleBar}>
            <AppInitializeError error={error()} />
          </AppLayout>
        }
      >
        {props.children}
      </Show>
    </Show>
  );
};
