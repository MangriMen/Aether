import { createSignal, onMount, Show } from 'solid-js';
import type { Component, JSX } from 'solid-js';
import {
  initializeApp,
  initializeResources,
  showWindow,
  useSetupListeners,
} from '../../lib';
import { AppInitializeError } from './AppInitializeError';
import { initializePlugins } from '@/5_entities/minecraft';
import { refetchPlugins } from '@/entities/plugins';

export type AppProviderProps = {
  children?: JSX.Element;
};

export const AppProvider: Component<AppProviderProps> = (props) => {
  const [isInitialized, setIsInitialized] = createSignal(false);
  const [initializeError, setInitializeError] = createSignal<
    string | undefined
  >();

  const init = async () => {
    setIsInitialized(false);
    setInitializeError(undefined);

    try {
      await initializeApp();
      await initializeResources();
    } catch (e) {
      console.error(e);
      if (e instanceof Error) {
        setInitializeError(e.message);
      }
    }

    setIsInitialized(true);
    await showWindow();

    if (!initializeError()) {
      loadPlugins();
    }
  };

  const loadPlugins = async () => {
    await initializePlugins();
    await refetchPlugins();
  };

  onMount(() => init());

  useSetupListeners();

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
