import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import { listProcessRaw, getInstanceProcessRaw } from '../rawApi';
import { PROCESS_QUERY_KEYS } from './process_query_keys';

export const useProcessList = () => {
  return useQuery(() => ({
    queryKey: PROCESS_QUERY_KEYS.LIST(),
    queryFn: listProcessRaw,
  }));
};

export const useInstanceProcess = (id: Accessor<string>) => {
  return useQuery(() => ({
    queryKey: PROCESS_QUERY_KEYS.BY_INSTANCE(id()),
    queryFn: () => getInstanceProcessRaw(id()),
    enabled: !!id,
  }));
};
