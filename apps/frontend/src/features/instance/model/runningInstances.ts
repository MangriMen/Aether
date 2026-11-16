import { createStore, reconcile, unwrap } from 'solid-js/store';

import type { RunningProcessUpdate } from '@/features/process';

import type {
  LaunchState,
  RunningInstancesContextValue,
} from './runningInstancesContext';

export type RunningInstancesState =
  RunningInstancesContextValue[0]['instances'];

/**
 * What of a launch outlives the game starting: a call off was asked for, and the launch that
 * returns with a game stops it. A launch that goes on is over for the state as the game runs.
 */
const keptLaunch = (
  current: { launch?: LaunchState } | undefined,
): { launch?: LaunchState; isLoading: boolean } =>
  current?.launch === 'cancelling'
    ? { launch: 'cancelling', isLoading: true }
    : { isLoading: false };

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
      isRunning: true,
      ...keptLaunch(current),
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
    const current = instances[instanceId];

    next[instanceId] = {
      processId,
      isLoading: current?.isLoading ?? false,
      isRunning: true,
      ...(current?.launch === 'cancelling' ? { launch: 'cancelling' } : {}),
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

export type StopAction =
  | { kind: 'cancelLaunch' }
  | { kind: 'stopProcess'; processId: string };

/**
 * What stopping the instance does: stops its game, or, while it launches and has no game yet, calls
 * the launch off. Nothing while that is under way already.
 */
export const getStopAction = (
  running:
    | {
        processId?: string;
        isLoading: boolean;
        isRunning: boolean;
        launch?: LaunchState;
      }
    | undefined,
): StopAction | undefined => {
  const processId = running?.isRunning
    ? getStoppableProcessId(running)
    : undefined;

  if (processId !== undefined) {
    return { kind: 'stopProcess', processId };
  }

  return running?.launch === 'launching' && !running.isRunning
    ? { kind: 'cancelLaunch' }
    : undefined;
};

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
 * Applies a change of the launch state: the loading flag follows it, and a launch that is over
 * leaves what it left, as `applyLoadingChange` does.
 */
export const applyLaunchChange = (
  instances: RunningInstancesState,
  id: string,
  launch: LaunchState | undefined,
): RunningInstancesState => {
  const next = applyLoadingChange(instances, id, launch !== undefined);
  const entry = next[id];

  if (entry === undefined) {
    return next;
  }

  const { launch: _previous, ...rest } = entry;

  return { ...next, [id]: launch === undefined ? rest : { ...rest, launch } };
};

/**
 * Launches that are in progress and that this page knows nothing of (it was reloaded meanwhile):
 * they are taken as launching, and no call comes back to say that they are over.
 */
export const applyRecoveredLaunches = (
  instances: RunningInstancesState,
  ids: string[],
): RunningInstancesState => {
  const next = { ...instances };

  for (const id of ids) {
    if (next[id] === undefined) {
      next[id] = {
        processId: undefined,
        isLoading: true,
        isRunning: false,
        launch: 'launching',
      };
    }
  }

  return next;
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
    applyLaunchChange: (id: string, launch: LaunchState | undefined) =>
      apply((instances) => applyLaunchChange(instances, id, launch)),
    applyRecoveredLaunches: (ids: string[]) =>
      apply((instances) => applyRecoveredLaunches(instances, ids)),
    /** Ends the launches of `ids` that were recovered, if they still wait for a game. */
    applyEndedLaunches: (ids: string[]) =>
      apply((instances) =>
        ids.reduce(
          (current, id) =>
            current[id]?.launch === 'launching'
              ? applyLaunchChange(current, id, undefined)
              : current,
          instances,
        ),
      ),
  };
};
