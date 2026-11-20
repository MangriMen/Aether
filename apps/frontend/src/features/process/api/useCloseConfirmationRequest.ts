import { listen } from '@tauri-apps/api/event';
import { createSignal, onCleanup, onMount } from 'solid-js';

import { applicationCommands } from '@/platform/ipc';
import { logError } from '@/shared/log';

/** Sent by the backend when the window is closed while a game runs. */
const CLOSE_CONFIRMATION_EVENT = 'aether://close-confirmation-requested';

/**
 * The close of the window that the backend held because a game runs: `isRequested` is true
 * until the user agrees (`confirm` closes the window for good) or backs out (`cancel`).
 */
export const useCloseConfirmationRequest = () => {
  const [isRequested, setIsRequested] = createSignal(false);

  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    listen(CLOSE_CONFIRMATION_EVENT, () => setIsRequested(true))
      .then((fn) => {
        if (disposed) {
          fn();
          return;
        }

        unlisten = fn;
        // Told once the listening is certain: the backend holds a close only then.
        return applicationCommands.setCloseConfirmationReady(true);
      })
      .catch((err: unknown) => {
        logError('Failed to listen for close requests:', err);
      });

    onCleanup(() => {
      disposed = true;
      unlisten?.();
      applicationCommands.setCloseConfirmationReady(false).catch(logError);
    });
  });

  const confirm = () => {
    setIsRequested(false);
    applicationCommands.quitApp().catch((err: unknown) => {
      logError('Failed to close the launcher:', err);
    });
  };

  const cancel = () => setIsRequested(false);

  return { isRequested, confirm, cancel };
};
