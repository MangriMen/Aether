import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import { processCommands } from '@/platform/ipc';

import { processKeys } from './processQueryKeys';

export const useProcessList = () => {
  return useQuery(() => ({
    queryKey: processKeys.list(),
    queryFn: processCommands.list,
  }));
};

export const useInstanceProcess = (id: Accessor<string>) => {
  return useQuery(() => ({
    queryKey: processKeys.byInstance(id()),
    queryFn: () => processCommands.getByInstanceId(id()),
    enabled: !!id,
  }));
};
