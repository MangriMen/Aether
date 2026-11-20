import { createContext, useContext } from 'solid-js';

import type { Instance } from './instance';

type RunningInstancesInstanceData = {
  processId?: string;
  isLoading: boolean;
  isRunning: boolean;
  /** Started by an earlier launcher: the launcher follows the game but has no console output of it. */
  adopted?: boolean;
  /** A launch has the instance, and may be called off until the game starts. */
  isLaunching?: boolean;
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
