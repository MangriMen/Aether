import { createSignal, onMount, Show } from 'solid-js';
import type { Component, JSX } from 'solid-js';
import { initializeApp, showWindow } from '../../lib';
import { AppInitializeError } from './AppInitializeError';
import { initializePlugins } from '@/entities/minecraft';

export type AppInitializeGuardProps = { children?: JSX.Element };

export const AppInitializeGuard: Component<AppInitializeGuardProps> = (
  props,
) => {
  const [isInitialized, setIsInitialized] = createSignal(false);
  const [initializeError, setInitializeError] = createSignal<
    string | undefined
  >();

  const init = async () => {
    setIsInitialized(false);
    setInitializeError(undefined);

    try {
      await initializeApp();
    } catch (e) {
      console.error(e);
      if (e instanceof Error) {
        setInitializeError(e.message);
      }
    }

    setIsInitialized(true);
    await showWindow();

    try {
      if (!initializeError()) {
        await initializePlugins();
      }
    } catch (e) {
      console.error(e);
    }
  };

  onMount(() => init());

  return (
    <Show when={isInitialized()}>
      <Show
        when={!initializeError()}
        fallback={<AppInitializeError message={initializeError()} />}
      >
        {props.children}
      </Show>
    </Show>
  );
};
