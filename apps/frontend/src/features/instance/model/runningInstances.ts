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
 * Replaces what is known of running processes with a fresh list: a game the list does not have
 * is not running, whatever the events said. A launch that has no process yet is kept.
 */
export const applyProcessSnapshot = (
  instances: RunningInstancesState,
  processes: { instanceId: string; processId: string }[],
): RunningInstancesState => {
  const next: RunningInstancesState = {};

  for (const [id, current] of Object.entries(instances)) {
    if (current !== undefined && !current.isRunning) {
      next[id] = current;
    }
  }

  for (const { instanceId, processId } of processes) {
    next[instanceId] = {
      processId,
      isLoading: instances[instanceId]?.isLoading ?? false,
      isRunning: true,
    };
  }

  return next;
};

/**
 * The process to stop, if there is one: a running game is stopped whatever the install stage
 * says, a launch in progress has nothing to stop yet.
 */
export const getStoppableProcessId = (
  running: { processId?: string; isLoading: boolean } | undefined,
): string | undefined => (running?.isLoading ? undefined : running?.processId);

export type RemovalBlock = 'running' | 'launching' | 'notInstalled';

/**
 * Why an instance cannot be removed right now. Only a not fully installed one may be removed
 * by force; removing a game that runs or launches would pull its files from under it.
 */
export const getRemovalBlock = (
  installStage: string,
  running: { isLoading: boolean; isRunning: boolean } | undefined,
): RemovalBlock | undefined => {
  if (running?.isRunning) {
    return 'running';
  }

  if (running?.isLoading) {
    return 'launching';
  }

  return installStage === 'installed' ? undefined : 'notInstalled';
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
    applyProcessSnapshot: (
      processes: { instanceId: string; processId: string }[],
    ) => apply((instances) => applyProcessSnapshot(instances, processes)),
    applyLoadingChange: (id: string, isLoading: boolean) =>
      apply((instances) => applyLoadingChange(instances, id, isLoading)),
  };
};
