import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import { getInstanceProcessRaw, listProcessRaw } from '../rawApi';
import { PROCESS_QUERY_KEYS } from './process_query_keys';

export const useProcessList = () => {
  return useQuery(() => ({
    queryFn: listProcessRaw,
    queryKey: PROCESS_QUERY_KEYS.LIST(),
  }));
};

export const useInstanceProcess = (id: Accessor<string>) => {
  return useQuery(() => ({
    enabled: !!id,
    queryFn: () => getInstanceProcessRaw(id()),
    queryKey: PROCESS_QUERY_KEYS.BY_INSTANCE(id()),
  }));
};
