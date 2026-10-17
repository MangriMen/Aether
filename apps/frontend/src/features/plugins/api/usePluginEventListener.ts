import { useQueryClient } from '@tanstack/solid-query';

import { pluginsEvents, useEventListener } from '@/platform/ipc';
import { logDebug } from '@/shared/utils';

import { pluginsCache } from './pluginsCache';

export const usePluginEventListener = () => {
  const queryClient = useQueryClient();

  useEventListener(pluginsEvents.pluginEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    switch (e.payload.type) {
      case 'sync':
        pluginsCache.invalidate.all(queryClient);
        break;
      case 'add':
      case 'edit':
      case 'remove':
        pluginsCache.invalidate.full(queryClient, e.payload.plugin_id);
        break;
    }
  });
};
