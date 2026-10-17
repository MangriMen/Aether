import { isDebug, setIsDebug } from '@/shared/log';

export const AetherNamespace = '_AETHER_';

export const AetherNamespaceMethods = {
  toggleDebug: () =>
    // eslint-disable-next-line no-console
    console.debug(setIsDebug(!isDebug()) ? 'Debug enabled' : 'Debug disabled'),
};
