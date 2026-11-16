import { useQuery } from '@tanstack/solid-query';

import { pluginsCommands } from '@/platform/ipc';

import { pluginKeys } from './pluginKeys';

/**
 * Whether the core finds an `allowed_hosts` entry wide. The rule lives in the core; until it
 * answers, nothing is warned about.
 */
export const useIsWideAllowedHost = (entry: () => string) =>
  useQuery(() => ({
    queryKey: pluginKeys.wideHost(entry()),
    queryFn: () => pluginsCommands.isWideAllowedHost(entry()),
    enabled: entry().trim() !== '',
    staleTime: Infinity,
  }));
