import type { Component, JSX } from 'solid-js';

import { createSignal, onMount, Show } from 'solid-js';

import { initializePlugins } from '@/entities/minecraft';
import { debugError } from '@/shared/lib/log';

import { initializeApp, showWindow } from '../../lib';
import { AppInitializeError } from './AppInitializeError';

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
      debugError(e);
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
      debugError(e);
    }
  };

  onMount(() => init());

  return (
    <Show when={isInitialized()}>
      <Show
        fallback={<AppInitializeError message={initializeError()} />}
        when={!initializeError()}
      >
        {props.children}
      </Show>
    </Show>
  );
};
