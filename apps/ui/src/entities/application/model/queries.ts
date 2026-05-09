import { useMutation, useQueryClient } from '@tanstack/solid-query';

import { appSettingsCache } from '@/entities/settings/@x/application';

import { commands } from '../api';

export const useRecreateWindow = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: commands.recreateWindow,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
  }));
};
