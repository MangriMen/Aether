import { AetherNamespace, AetherNamespaceMethods } from '../model';

export const exposeWindowMethods = () => {
  window[AetherNamespace] = AetherNamespaceMethods;
};
