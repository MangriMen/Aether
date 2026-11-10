import { eventsCommands } from '@/platform/ipc';
import { logError } from '@/shared/log';

/** Asks the backend to call off the work of a bar; the bar's own events tell how it went. */
export const cancelProgress = async (progressBarId: string) => {
  try {
    await eventsCommands.cancelProgress(progressBarId);
  } catch (error) {
    logError('Failed to cancel progress:', error);
  }
};
