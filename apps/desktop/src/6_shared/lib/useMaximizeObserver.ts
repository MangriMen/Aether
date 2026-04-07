import { throttle } from '@solid-primitives/scheduled';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { createEffect, onCleanup } from 'solid-js';

import { updateMaximize } from '@/shared/model';

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

    const handleResizeThrottle = throttle(
      async () => updateMaximize(await appWindow.isMaximized()),
      throttleTimeout,
    );

    const unlistenResizeEvent = await appWindow.onResized(handleResizeThrottle);

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
