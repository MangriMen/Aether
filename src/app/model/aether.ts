import { isDebug, setIsDebug } from '@/shared/model/settings';

export const AetherNamespaceName = '_AETHER_';

export const AetherNamespaceObject = {
  toggleDebug: () => {
    setIsDebug(!isDebug());
  },
};
