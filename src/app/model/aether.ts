import { isDebug, setIsDebug } from '@/shared/model';

export const AetherNamespaceName = '_AETHER_';

export const AetherNamespaceObject = {
  toggleDebug: () =>
    console.info(setIsDebug(!isDebug()) ? 'Debug enabled' : 'Debug disabled'),
};
