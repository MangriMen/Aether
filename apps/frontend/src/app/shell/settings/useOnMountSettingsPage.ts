import { useQueryClient } from '@tanstack/solid-query';
import { onMount } from 'solid-js';

import { settingsCache } from '@/features/settings';

export const useOnMountSettingsPage = () => {
  const queryClient = useQueryClient();

  onMount(() => {
    void settingsCache.prefetch.maxRam(queryClient);
  });
};
