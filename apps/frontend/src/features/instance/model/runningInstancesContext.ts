import { createContext, useContext } from 'solid-js';

import type { Instance } from './instance';

/**
 * A launch that has no game yet. The user may call it off while it is `launching`; a call off
 * is `cancelling` until the launch ends, or until the game, which started all the same, is stopped.
 */
export type LaunchState = 'cancelling' | 'launching';

type RunningInstancesInstanceData = {
  processId?: string;
  isLoading: boolean;
  isRunning: boolean;
  launch?: LaunchState;
};

export type RunningInstancesContextValue = [
  {
    instances: Record<Instance['id'], RunningInstancesInstanceData | undefined>;
  },
  {
    get: (
      context: RunningInstancesContextValue[0],
      id: Instance['id'],
    ) => RunningInstancesInstanceData | undefined;
    setIsLoading: (id: Instance['id'], value: boolean) => void;
    /** Sets the state of the launch of the instance; `undefined` when the launch is over. */
    setLaunch: (id: Instance['id'], value: LaunchState | undefined) => void;
    /** Re-reads the running processes, so the state does not rely on events alone. */
    refresh: () => Promise<void>;
  },
];

const DEFAULT_VALUE: RunningInstancesContextValue = [
  { instances: {} },
  {
    get: (_context: RunningInstancesContextValue[0], _id: Instance['id']) =>
      undefined,
    setIsLoading: (_id: Instance['id'], _value: boolean) => undefined,
    setLaunch: (_id: Instance['id'], _value: LaunchState | undefined) =>
      undefined,
    refresh: () => Promise.resolve(),
  },
];

export const RunningInstancesContext = createContext(DEFAULT_VALUE);

export const useRunningInstancesContext = () => {
  const value = useContext(RunningInstancesContext);

  if (!value) {
    throw new Error('Missing context Provider');
  }

  return value;
};
