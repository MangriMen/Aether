import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Component, createEffect, onCleanup } from 'solid-js';

import { updateMaximize } from '@/shared/model';

export const WindowObserver: Component<Record<never, never>> = () => {
  const appWindow = getCurrentWebviewWindow();
  let unlistenFn: VoidFunction | undefined = undefined;

  createEffect(() => {
    (async () => {
      if (unlistenFn) {
        unlistenFn();
      }

      unlistenFn = await appWindow.onResized(async () =>
        updateMaximize(await appWindow.isMaximized()),
      );
    })();

    onCleanup(() => unlistenFn && unlistenFn());
  });

  return null;
};
