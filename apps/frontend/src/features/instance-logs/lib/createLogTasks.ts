import { logError } from '@/shared/log';

/**
 * Runs the work on a log view one task after another, and tells which work is out of date:
 * a task that was started for an earlier generation must not touch what a later one shows.
 */
export const createLogTasks = () => {
  let generation = 0;
  let chain: Promise<void> = Promise.resolve();

  const enqueue = (task: () => Promise<void>) => {
    chain = chain.then(task).catch((err: unknown) => {
      logError('[LOGS] failed to update the log view', err);
    });

    return chain;
  };

  return {
    enqueue,
    /** The generation work started now belongs to. */
    current: () => generation,
    /** Makes all work started so far out of date, and returns the new generation. */
    next: () => {
      generation += 1;

      return generation;
    },
  };
};

export type LogTasks = ReturnType<typeof createLogTasks>;
