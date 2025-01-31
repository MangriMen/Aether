import type { UnlistenFn } from '@tauri-apps/api/event';
import type { Component, JSX } from 'solid-js';
import { createEffect, onCleanup } from 'solid-js';
import { createStore } from 'solid-js/store';

import type { Instance, RunningInstancesContextValue } from '../model';
import { RunningInstancesContext, ProcessPayloadType } from '../model';
import { listenProcess } from '../api';

export type RunningInstancesContextProps = { children?: JSX.Element };

export const RunningInstancesProvider: Component<
  RunningInstancesContextProps
> = (props) => {
  const [contextValue, setContextValue] = createStore<
    RunningInstancesContextValue[0]
  >({
    instances: {},
  });

  const context: RunningInstancesContextValue = [
    contextValue,
    {
      get: (context: RunningInstancesContextValue[0], id: Instance['id']) => {
        return context.instances[id];
      },
      setIsLoading: (id, value) => {
        if (!context[1].get(contextValue, id)) {
          setContextValue('instances', id, () => ({
            payload: undefined,
            isLoading: value,
            isRunning: false,
          }));
          return;
        }
        setContextValue('instances', id, 'isLoading', () => value);
      },
    },
  ];

  let processListenerUnlistenFn: UnlistenFn | undefined;

  const stopProcessListener = () => processListenerUnlistenFn?.();

  const startProcessListener = () =>
    listenProcess((e) => {
      const isLaunched = e.payload.event === ProcessPayloadType.Launched;

      setContextValue('instances', (instances) => ({
        ...instances,
        [e.payload.instanceId]: {
          payload: e.payload,
          isLoading: false,
          isRunning: isLaunched,
        },
      }));
    });

  createEffect(() => {
    startProcessListener().then((unlistenFn) => {
      processListenerUnlistenFn = unlistenFn;
    });
  });

  onCleanup(() => {
    stopProcessListener();
  });

  return (
    <RunningInstancesContext.Provider value={context}>
      {props.children}
    </RunningInstancesContext.Provider>
  );
};
