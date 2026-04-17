import { getCurrentWindow } from '@tauri-apps/api/window';

import { applicationCommands } from '@/shared/api';

import { AetherNamespace, AetherNamespaceMethods } from '../model';

export const initializeWindowMethods = () => {
  window[AetherNamespace] = AetherNamespaceMethods;
};

export const initializeApp = async () => {
  await applicationCommands.initializeState();
  initializeWindowMethods();
};

export const showWindow = async () => {
  await getCurrentWindow().show();
};
