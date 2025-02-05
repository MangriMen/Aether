import { initializeState } from '@/entities/minecrafts';

import { AetherNamespaceName, AetherNamespaceObject } from '../model';
import { getCurrentWindow } from '@tauri-apps/api/window';

export const initializeHandlers = () => {
  window[AetherNamespaceName] = AetherNamespaceObject;
};

export const initializeApp = async () => {
  initializeHandlers();
  await initializeState();
  await getCurrentWindow().show();
};
