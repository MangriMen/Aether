import { initializeState } from '@/entities/minecraft';

import { AetherNamespace, AetherNamespaceMethods } from '../model';
import { getCurrentWindow } from '@tauri-apps/api/window';

export const initializeWindowMethods = () => {
  window[AetherNamespace] = AetherNamespaceMethods;
};

export const initializeApp = async () => {
  await initializeState();
  initializeWindowMethods();
};

export const showWindow = async () => {
  await getCurrentWindow().show();
};
