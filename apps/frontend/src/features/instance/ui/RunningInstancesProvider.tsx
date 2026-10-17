import type { Component, JSX } from 'solid-js';

import { createStore } from 'solid-js/store';

import { useProcessEventListener } from '@/features/process';

import type { Instance } from '../model/instance';
import type { RunningInstancesContextValue } from '../model/runningInstancesContext';

import { RunningInstancesContext } from '../model/runningInstancesContext';

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

  useProcessEventListener((event) => {
    setContextValue('instances', (instances) => ({
      ...instances,
      [event.instanceId]: {
        payload: event,
        isLoading: false,
        isRunning: event.event === 'launched',
      },
    }));
  });

  return (
    <RunningInstancesContext.Provider value={context}>
      {props.children}
    </RunningInstancesContext.Provider>
  );
};
