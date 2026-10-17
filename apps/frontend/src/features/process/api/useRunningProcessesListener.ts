import { onMount } from 'solid-js';

import { processCommands } from '@/platform/ipc';

import { useProcessEventListener } from './useProcessEventListener';

export type RunningProcessUpdate = {
  instanceId: string;
  processId: string;
  isRunning: boolean;
};

/**
 * Running game processes: the ones already running on mount (the webview may have been
 * reloaded while a game runs), then live process events.
 */
export const useRunningProcessesListener = (
  callback: (update: RunningProcessUpdate) => void,
) => {
  // Instances updated by live events while the initial snapshot is loading;
  // the snapshot must not overwrite them. Not needed once it is applied.
  let liveInstanceIds: Set<string> | undefined = new Set<string>();

  useProcessEventListener((event) => {
    liveInstanceIds?.add(event.instanceId);
    callback({
      instanceId: event.instanceId,
      processId: event.processId,
      isRunning: event.event === 'launched',
    });
  });

  const fetchProcesses = async () => {
    try {
      const processes = await processCommands.list();

      for (const process of processes) {
        if (liveInstanceIds?.has(process.instance_id)) {
          continue;
        }

        callback({
          instanceId: process.instance_id,
          processId: process.uuid,
          isRunning: true,
        });
      }
    } catch {
      /* empty */
    } finally {
      liveInstanceIds = undefined;
    }
  };

  onMount(fetchProcesses);
};
