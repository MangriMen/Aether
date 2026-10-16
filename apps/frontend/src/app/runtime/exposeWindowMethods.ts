import { AetherNamespace, AetherNamespaceMethods } from './aether';

export const exposeWindowMethods = () => {
  window[AetherNamespace] = AetherNamespaceMethods;
};
