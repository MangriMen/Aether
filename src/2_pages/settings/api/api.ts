import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { getMicaRaw, setMicaRaw } from './rawApi';

export const useMica = () =>
  useQuery(() => ({
    queryKey: ['mica'],
    queryFn: getMicaRaw,
  }));

export const useSetMica = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: setMicaRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mica'] });
    },
  }));
};
