import { useMutation, useQueryClient } from '@tanstack/solid-query';

import { appSettingsCache } from '@/entities/settings/@x/application';

import { applicationCommands } from '../api';

export const useRecreateWindow = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: applicationCommands.recreateWindow,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
  }));
};
