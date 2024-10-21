import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { createSignal } from 'solid-js';

const [isMaximizedSig, setIsMaximizedSig] = createSignal(false);

export const isMaximized = isMaximizedSig;

export const toggleMaximize = async () => {
  const appWindow = getCurrentWebviewWindow();

  const isMaximized = await appWindow.isMaximized();
  isMaximized ? appWindow.unmaximize() : appWindow.maximize();

  setIsMaximizedSig(!isMaximized);
};
