import { throttle } from '@solid-primitives/scheduled';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { Component } from 'solid-js';
import { createEffect, onCleanup } from 'solid-js';

import { updateMaximize } from '@/shared/model';

export const WindowObserver: Component<Record<never, never>> = () => {
  const appWindow = getCurrentWebviewWindow();
  let unlistenFn: VoidFunction | undefined = undefined;

  createEffect(() => {
    (async () => {
      if (unlistenFn) {
        unlistenFn();
      }

      const handleResizeThrottle = throttle(
        async () => updateMaximize(await appWindow.isMaximized()),
        400,
      );

      const unlistenEvent = await appWindow.onResized(handleResizeThrottle);

      unlistenFn = async () => {
        unlistenEvent();
        handleResizeThrottle.clear();
      };
    })();

    onCleanup(() => unlistenFn && unlistenFn());
  });

  return null;
};
