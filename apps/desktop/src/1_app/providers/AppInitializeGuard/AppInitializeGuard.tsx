import type { Component, JSX } from 'solid-js';

import { createSignal, onMount, Show } from 'solid-js';

import { AppLayout } from '@/app/layouts/AppLayout';
import { setupApp } from '@/app/model';
import { BaseTitleBar } from '@/widgets/app-titlebar/ui/BaseTitleBar';

import { AppInitializeError } from './AppInitializeError';

export type AppInitializeGuardProps = { children?: JSX.Element };

export const AppInitializeGuard: Component<AppInitializeGuardProps> = (
  props,
) => {
  const [isReady, setIsReady] = createSignal(false);
  const [error, setError] = createSignal<string | undefined>();

  const handleSetup = async () => {
    setIsReady(false);
    setError(undefined);

    try {
      await setupApp();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
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
