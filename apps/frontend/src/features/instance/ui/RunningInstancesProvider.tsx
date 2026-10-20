import type { Component, JSX } from 'solid-js';

import { createStore } from 'solid-js/store';

import { useRunningProcessesListener } from '@/features/process';

import type { Instance } from '../model/instance';
import type { RunningInstancesContextValue } from '../model/runningInstancesContext';

import { applyRunningProcessUpdate } from '../model/runningInstances';
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
        const current = context[1].get(contextValue, id);

        if (!current) {
          // Nothing to clear: the process ended and its entry is gone.
          if (!value) {
            return;
          }

          setContextValue('instances', id, () => ({
            processId: undefined,
            isLoading: value,
            isRunning: false,
          }));
          return;
        }

        // A launch that failed leaves nothing to show: the entry goes with it.
        if (!value && !current.isRunning) {
          setContextValue('instances', (instances) => {
            const rest = { ...instances };

            delete rest[id];

            return rest;
          });
          return;
        }

        setContextValue('instances', id, 'isLoading', () => value);
      },
    },
  ];

  useRunningProcessesListener((update) => {
    setContextValue('instances', (instances) =>
      applyRunningProcessUpdate(instances, update),
    );
  });

  return (
    <RunningInstancesContext.Provider value={context}>
      {props.children}
    </RunningInstancesContext.Provider>
  );
};
