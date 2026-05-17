import { throttle } from '@solid-primitives/scheduled';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { createEffect, onCleanup } from 'solid-js';

import { updateMaximize } from '../model';

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

  const initializeResizeObserver = async () => {
    unlistenResize?.();

    const handleResize = async () =>
      updateMaximize(await appWindow.isMaximized());

    const handleResizeThrottle = throttle(handleResize, throttleTimeout);

    const unlistenResizeEvent = await appWindow.onResized(handleResizeThrottle);

    handleResize();

    unlistenResize = () => {
      unlistenResizeEvent();
      handleResizeThrottle.clear();
    };
  };

  createEffect(() => {
    initializeResizeObserver();
    onCleanup(() => unlistenResize?.());
  });
};
