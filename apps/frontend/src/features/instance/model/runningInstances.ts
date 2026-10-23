import { createStore, reconcile, unwrap } from 'solid-js/store';

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

/**
 * Applies a change of the loading flag. A launch that failed (no process, not running) and a
 * process that is already gone leave no entry behind.
 */
export const applyLoadingChange = (
  instances: RunningInstancesState,
  id: string,
  isLoading: boolean,
): RunningInstancesState => {
  const current = instances[id];

  if (current === undefined) {
    if (!isLoading) {
      return instances;
    }

    return {
      ...instances,
      [id]: { processId: undefined, isLoading: true, isRunning: false },
    };
  }

  if (!isLoading && !current.isRunning) {
    const rest = { ...instances };

    delete rest[id];

    return rest;
  }

  return { ...instances, [id]: { ...current, isLoading } };
};

/**
 * The store of running instances. A store setter merges an object into the state, so a
 * removed entry would stay; reconcile replaces the map and drops the keys that are gone.
 */
export const createRunningInstancesStore = () => {
  const [state, setState] = createStore<RunningInstancesContextValue[0]>({
    instances: {},
  });

  const apply = (
    change: (instances: RunningInstancesState) => RunningInstancesState,
  ) => {
    setState('instances', reconcile(change(unwrap(state.instances))));
  };

  return {
    state,
    applyProcessUpdate: (update: RunningProcessUpdate) =>
      apply((instances) => applyRunningProcessUpdate(instances, update)),
    applyLoadingChange: (id: string, isLoading: boolean) =>
      apply((instances) => applyLoadingChange(instances, id, isLoading)),
  };
};
