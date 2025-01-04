import { initializeState } from '@/entities/minecraft';

import { AetherNamespaceName, AetherNamespaceObject } from '../model';

export const initializeHandlers = () => {
  window[AetherNamespaceName] = AetherNamespaceObject;
};

export const initializeApp = async () => {
  await initializeState();
  initializeHandlers();
};
