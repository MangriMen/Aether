import { IS_DEBUG_KEY, isDebug, setIsDebug } from '@/shared/model';

export const AetherNamespaceName = '_AETHER_';

export const AetherNamespaceObject = {
  toggleDebug: () => {
    const newValue = !isDebug();
    localStorage.setItem(IS_DEBUG_KEY, JSON.stringify(newValue));
    setIsDebug(newValue);
  },
};
