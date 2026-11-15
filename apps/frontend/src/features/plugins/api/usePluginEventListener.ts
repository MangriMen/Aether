import { useQueryClient } from '@tanstack/solid-query';
import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window';

import { pluginsEvents, useEventListener } from '@/platform/ipc';
import { logDebug } from '@/shared/log';

import { invalidateCommandApprovals } from './commandApprovalQueries';
import { pluginsCache } from './pluginsCache';

export const usePluginEventListener = () => {
  const queryClient = useQueryClient();

  // The backend loads plugins on its own at startup, possibly before this listener exists.
  pluginsCache.invalidate.all(queryClient);
  // A question may already wait: the backend runs plugins before this listener exists.
  void invalidateCommandApprovals(queryClient);

  useEventListener(pluginsEvents.pluginEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    switch (e.payload.type) {
      case 'sync':
        pluginsCache.invalidate.all(queryClient);
        break;
      case 'edit':
        pluginsCache.invalidate.full(queryClient, e.payload.plugin_id);
        break;
      case 'commandApprovalRequested':
        // The window may be behind another or minimised, and the plugin waits for the answer.
        getCurrentWindow()
          .requestUserAttention(UserAttentionType.Informational)
          .catch(() => undefined);
        void invalidateCommandApprovals(queryClient);
        break;
      case 'commandApprovalResolved':
        void invalidateCommandApprovals(queryClient);
        break;
    }
  });
};
