import { useQueryClient } from '@tanstack/solid-query';
import { onMount } from 'solid-js';

import { settingsCache } from '../../../entities/settings/model/settingsQueries/cache';

export const useOnMountSettingsPage = () => {
  const queryClient = useQueryClient();

  onMount(() => {
    settingsCache.prefetch.maxRam(queryClient);
  });
};
