import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { createSignal } from 'solid-js';

const [isMaximizedSignal, setIsMaximizedSignal] = createSignal(false);

export const isMaximized = isMaximizedSignal;

/**
 * Toggles the maximized state of the current webview window.
 * If the window is maximized, it will be unmaximized, and vice versa.
 * Updates the internal maximized signal to reflect the new state.
 */
export const toggleMaximize = async () => {
  const appWindow = getCurrentWebviewWindow();

  const isMaximized = await appWindow.isMaximized();
  if (isMaximized) {
    appWindow.unmaximize();
  } else {
    appWindow.maximize();
  }
};

/**
 * Updates the internal state of the maximized signal without changing the
 * maximized state of the window.
 *
 * @param isMaximized New value of the maximized signal
 */
export const updateMaximize = (isMaximized: boolean) => {
  setIsMaximizedSignal(isMaximized);
};
