import { initializeState } from '@/entities/minecrafts';

import { AetherNamespace, AetherNamespaceMethods } from '../model';
import { getCurrentWindow } from '@tauri-apps/api/window';

export const initializeWindowMethods = () => {
  window[AetherNamespace] = AetherNamespaceMethods;
};

export const initializeApp = async () => {
  await initializeState();
  initializeWindowMethods();
  await getCurrentWindow().show();
};
