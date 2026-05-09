import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import { commands } from '../api';
import { processKeys } from './queryKeys';

export const useProcessList = () => {
  return useQuery(() => ({
    queryKey: processKeys.list(),
    queryFn: commands.list,
  }));
};

export const useInstanceProcess = (id: Accessor<string>) => {
  return useQuery(() => ({
    queryKey: processKeys.byInstance(id()),
    queryFn: () => commands.getByInstanceId(id()),
    enabled: !!id,
  }));
};
