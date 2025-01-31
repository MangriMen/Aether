import { updateMaximize } from '@/shared/model';
import { throttle } from '@solid-primitives/scheduled';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { createEffect, onCleanup } from 'solid-js';

const RESIZE_MAXIMIZE_THROTTLE_TIMEOUT = 400;

export const useMaximizeObserver = (
  throttleTimeout = RESIZE_MAXIMIZE_THROTTLE_TIMEOUT,
) => {
  const appWindow = getCurrentWebviewWindow();

  let unlistenResize: VoidFunction | undefined = undefined;

  createEffect(() => {
    (async () => {
      unlistenResize?.();

      const handleResizeThrottle = throttle(
        async () => updateMaximize(await appWindow.isMaximized()),
        throttleTimeout,
      );

      const unlistenResizeEvent =
        await appWindow.onResized(handleResizeThrottle);

      unlistenResize = async () => {
        unlistenResizeEvent();
        handleResizeThrottle.clear();
      };
    })();

    onCleanup(() => unlistenResize?.());
  });
};
