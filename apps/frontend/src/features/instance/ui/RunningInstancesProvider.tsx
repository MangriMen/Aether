import type { Component, JSX } from 'solid-js';

import { onCleanup, onMount } from 'solid-js';

import { useRunningProcessesListener } from '@/features/process';
import { instanceCommands, processCommands } from '@/platform/ipc';
import { logError } from '@/shared/log';

import type { Instance } from '../model/instance';
import type { RunningInstancesContextValue } from '../model/runningInstancesContext';

import { createRunningInstancesStore } from '../model/runningInstances';
import { RunningInstancesContext } from '../model/runningInstancesContext';

/** How often a launch that this page did not start is asked about, until it is over. */
const RECOVERED_LAUNCH_POLL_MS = 1000;

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
      setLaunch: store.applyLaunchChange,
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

  // The page may have been reloaded while a launch waits for its pre-launch hook: the launch has
  // no promise here to tell that it is over, so it is asked about until it is.
  onMount(() => {
    let disposed = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    // The launches that were found on mount and are not over yet; `undefined` until then.
    let pending: string[] | undefined;

    const poll = async () => {
      try {
        const launching = await instanceCommands.listLaunching();

        if (disposed) {
          return;
        }

        if (pending === undefined) {
          store.applyRecoveredLaunches(launching);
          pending = launching;
        } else {
          store.applyEndedLaunches(
            pending.filter((id) => !launching.includes(id)),
          );
          pending = pending.filter((id) => launching.includes(id));
        }
      } catch (error) {
        logError('Failed to list launching instances:', error);
      }

      if (pending?.length && !disposed) {
        timer = setTimeout(poll, RECOVERED_LAUNCH_POLL_MS);
      }
    };

    void poll();

    onCleanup(() => {
      disposed = true;
      clearTimeout(timer);
    });
  });

  return (
    <RunningInstancesContext.Provider value={context}>
      {props.children}
    </RunningInstancesContext.Provider>
  );
};
