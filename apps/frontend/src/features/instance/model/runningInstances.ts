import type { RunningProcessUpdate } from '@/features/process';

import type { RunningInstancesContextValue } from './runningInstancesContext';

export type RunningInstancesState =
  RunningInstancesContextValue[0]['instances'];

/**
 * Applies a process update to the running instances. A finished process leaves no entry
 * behind: the instance is not running and nothing is known about it any more.
 */
export const applyRunningProcessUpdate = (
  instances: RunningInstancesState,
  update: RunningProcessUpdate,
): RunningInstancesState => {
  const current = instances[update.instanceId];

  if (!update.isRunning) {
    // A late event of a process that was replaced by a newer one must not drop the newer one,
    // nor a launch that has no process yet.
    if (current === undefined || current.processId !== update.processId) {
      return instances;
    }

    const rest = { ...instances };

    delete rest[update.instanceId];

    return rest;
  }

  return {
    ...instances,
    [update.instanceId]: {
      processId: update.processId,
      isLoading: false,
      isRunning: true,
    },
  };
};
