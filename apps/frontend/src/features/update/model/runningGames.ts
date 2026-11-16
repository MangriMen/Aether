/**
 * The names of the games that run, in the order of the instance list. An instance the list does
 * not know is named by its id: the game runs all the same.
 */
export const getRunningGameNames = (
  running: Record<string, { isRunning: boolean } | undefined>,
  instances: readonly { id: string; name: string }[] | undefined,
): string[] =>
  Object.entries(running)
    .filter(([, state]) => state?.isRunning)
    .map(
      ([id]) => instances?.find((instance) => instance.id === id)?.name ?? id,
    );
