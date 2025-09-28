import { useQueryClient } from '@tanstack/solid-query';
import { onMount } from 'solid-js';

import { prefetchMaxRam } from '@/entities/settings';

export const useOnMountSettingsPage = () => {
  const queryClient = useQueryClient();

  onMount(() => {
    prefetchMaxRam(queryClient);
  });
};
