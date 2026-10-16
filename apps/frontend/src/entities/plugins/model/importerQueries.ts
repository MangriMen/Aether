import { useQuery, type QueryClient } from '@tanstack/solid-query';

import { instanceCommands } from '@/platform/ipc';

import { pluginKeys } from './queryKeys';

// Importers are capabilities of running plugins; the command lives in the instance backend.
export const useImporters = () =>
  useQuery(() => ({
    queryKey: pluginKeys.importerList(),
    queryFn: instanceCommands.listImporters,
  }));

export const invalidateImporters = (queryClient: QueryClient) =>
  queryClient.invalidateQueries({
    queryKey: pluginKeys.importers(),
  });
