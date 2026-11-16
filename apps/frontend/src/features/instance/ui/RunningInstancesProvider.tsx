import type { Component, JSX } from 'solid-js';

import { useRunningProcessesListener } from '@/features/process';
import { processCommands } from '@/platform/ipc';
import { logError } from '@/shared/log';

import type { Instance } from '../model/instance';
import type { RunningInstancesContextValue } from '../model/runningInstancesContext';

import { createRunningInstancesStore } from '../model/runningInstances';
import { RunningInstancesContext } from '../model/runningInstancesContext';

export type RunningInstancesContextProps = { children?: JSX.Element };

export const RunningInstancesProvider: Component<
  RunningInstancesContextProps
> = (props) => {
  const store = createRunningInstancesStore();

  const context: RunningInstancesContextValue = [
    store.state,
    {
      get: (context: RunningInstancesContextValue[0], id: Instance['id']) => {
        return context.instances[id];
      },
      setIsLoading: store.applyLoadingChange,
      refresh: async () => {
        try {
          const processes = await processCommands.list();

          store.applyProcessSnapshot(
            processes.map((p) => ({
              instanceId: p.instance_id,
              processId: p.uuid,
            })),
          );
        } catch (error) {
          logError('Failed to list running processes:', error);
        }
      },
    },
  ];

  useRunningProcessesListener(store.applyProcessUpdate);

  return (
    <RunningInstancesContext.Provider value={context}>
      {props.children}
    </RunningInstancesContext.Provider>
  );
};
