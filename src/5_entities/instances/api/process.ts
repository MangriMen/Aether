import { useQuery } from '@tanstack/solid-query';
import { listProcessRaw, getInstanceProcessRaw } from './rawApi';
import { QUERY_KEYS } from './query_keys';

export const useProcessList = () => {
  return useQuery(() => ({
    queryKey: [...QUERY_KEYS.INSTANCE.PROCESS.SELF(), 'list'],
    queryFn: listProcessRaw,
  }));
};

export const useInstanceProcess = (id: string) => {
  return useQuery(() => ({
    queryKey: QUERY_KEYS.INSTANCE.PROCESS.BY_INSTANCE(id),
    queryFn: () => getInstanceProcessRaw(id),
    enabled: !!id, // Только если ID существует
  }));
};
