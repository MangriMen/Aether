import { useQueryClient } from '@tanstack/solid-query';

import { pluginsEvents, useEventListener } from '@/platform/ipc';
import { logDebug } from '@/shared/log';

import { pluginsCache } from './pluginsCache';

export const usePluginEventListener = () => {
  const queryClient = useQueryClient();

  // The backend loads plugins on its own at startup, possibly before this listener exists.
  pluginsCache.invalidate.all(queryClient);

  useEventListener(pluginsEvents.pluginEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    switch (e.payload.type) {
      case 'sync':
        pluginsCache.invalidate.all(queryClient);
        break;
      case 'edit':
        pluginsCache.invalidate.full(queryClient, e.payload.plugin_id);
        break;
    }
  });
};
