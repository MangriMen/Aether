import { prefetchMaxRam } from '@/entities/settings';
import { useQueryClient } from '@tanstack/solid-query';
import { onMount } from 'solid-js';

export const useSettingsPagePrefetch = () => {
  const queryClient = useQueryClient();

  onMount(() => {
    prefetchMaxRam(queryClient);
  });
};
