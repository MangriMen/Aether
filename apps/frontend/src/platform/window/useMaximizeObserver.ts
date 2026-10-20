import { throttle } from '@solid-primitives/scheduled';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { onCleanup, onMount } from 'solid-js';

import { logError } from '@/shared/log';

import { updateMaximize } from './window';

const RESIZE_MAXIMIZE_THROTTLE_TIMEOUT = 400;

/**
 * Observes window maximize state and notifies about it.
 *
 * @param {number} [throttleTimeout] - Debounce timeout in ms.
 * @returns {void}
 */
export const useMaximizeObserver = (
  throttleTimeout = RESIZE_MAXIMIZE_THROTTLE_TIMEOUT,
) => {
  const appWindow = getCurrentWebviewWindow();
  let unlistenResize: VoidFunction | undefined;
  // `onResized` resolves later; a cleanup that comes first must still remove the listener.
  let disposed = false;

  const initializeResizeObserver = async () => {
    const handleResize = async () =>
      updateMaximize(await appWindow.isMaximized());

    const handleResizeThrottle = throttle(handleResize, throttleTimeout);

    const unlistenResizeEvent = await appWindow.onResized(handleResizeThrottle);

    if (disposed) {
      unlistenResizeEvent();
      handleResizeThrottle.clear();
      return;
    }

    handleResize().catch((error: unknown) => {
      logError('Failed to read the maximize state:', error);
    });

    unlistenResize = () => {
      unlistenResizeEvent();
      handleResizeThrottle.clear();
    };
  };

  onMount(() => {
    initializeResizeObserver().catch((error: unknown) => {
      logError('Failed to observe window resize:', error);
    });
  });

  onCleanup(() => {
    disposed = true;
    unlistenResize?.();
  });
};
