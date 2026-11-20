import { onMount } from 'solid-js';

import { processCommands } from '@/platform/ipc';
import { logError } from '@/shared/log';

import { getRunningState } from '../model/processEventEffects';
import { useReadyProcessEventListener } from './useProcessEventListener';

export type RunningProcessUpdate = {
  instanceId: string;
  processId: string;
  isRunning: boolean;
  /** Started by an earlier launcher. */
  adopted?: boolean;
};

/**
 * Running game processes: the ones already running on mount (the webview may have been
 * reloaded while a game runs), then live process events.
 *
 * The listener is registered before the snapshot is taken: an event that comes between the two
 * would otherwise be lost, and a game that finished then would stay as a running ghost.
 */
export const useRunningProcessesListener = (
  callback: (update: RunningProcessUpdate) => void,
) => {
  // Instances updated by live events while the initial snapshot is loading;
  // the snapshot may be older than they are and must not overwrite them.
  // Not needed once it is applied.
  let liveInstanceIds: Set<string> | undefined = new Set<string>();

  const listening = useReadyProcessEventListener((event) => {
    const isRunning = getRunningState(event);

    if (isRunning === undefined) {
      return;
    }

    liveInstanceIds?.add(event.instanceId);
    callback({
      instanceId: event.instanceId,
      processId: event.processId,
      isRunning,
      adopted: event.event === 'adopted',
    });
  });

  const fetchProcesses = async () => {
    try {
      // Listening may have failed; the snapshot is still better than nothing then.
      await listening;

      const processes = await processCommands.list();

      for (const process of processes) {
        if (liveInstanceIds?.has(process.instanceId)) {
          continue;
        }

        callback({
          instanceId: process.instanceId,
          processId: process.uuid,
          isRunning: true,
          adopted: process.adopted,
        });
      }
    } catch (error) {
      // Later process events repair the state, but a running game may show as stopped until then.
      logError('Failed to list running processes:', error);
    } finally {
      liveInstanceIds = undefined;
    }
  };

  onMount(() => {
    void fetchProcesses();
  });
};
