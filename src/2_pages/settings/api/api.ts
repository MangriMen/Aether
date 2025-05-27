import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { QUERY_KEYS } from './queryKeys';
import { getAppSettingsRaw, updateAppSettingsRaw } from './rawApi';

export const useAppSettings = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.GET(),
    queryFn: getAppSettingsRaw,
  }));

export const useUpdateAppSettings = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: updateAppSettingsRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.GET() });
    },
  }));
};
