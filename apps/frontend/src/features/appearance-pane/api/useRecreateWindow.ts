import { useMutation, useQueryClient } from '@tanstack/solid-query';

import { appSettingsCache } from '@/features/settings';
import { applicationCommands } from '@/platform/ipc';

export const useRecreateWindow = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: applicationCommands.recreateWindow,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
  }));
};
