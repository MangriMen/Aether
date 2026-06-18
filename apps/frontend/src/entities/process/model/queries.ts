import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import { processCommands } from '../api';
import { processKeys } from './queryKeys';

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
