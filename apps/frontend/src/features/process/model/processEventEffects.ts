import type { ProcessEvent } from './processEvent';

/** Only these events change whether a game is running; any other kind must leave it alone. */
export const getRunningState = (event: ProcessEvent): boolean | undefined => {
  switch (event.event) {
    case 'launched':
      return true;
    case 'finished':
      return false;
    default:
      return undefined;
  }
};
