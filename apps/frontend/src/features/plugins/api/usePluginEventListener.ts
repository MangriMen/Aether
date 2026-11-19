import { useQueryClient } from '@tanstack/solid-query';
import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window';

import { pluginsEvents, useEventListener } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';
import { logDebug, logError } from '@/shared/log';

import {
  closeCommandNotAnswered,
  showCommandNotAnswered,
} from '../ui/showCommandNotAnswered';
import { invalidateCommandApprovals } from './commandApprovalQueries';
import { pluginsCache } from './pluginsCache';

export const usePluginEventListener = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  // The backend loads plugins on its own at startup, possibly before this listener exists.
  void pluginsCache.invalidate.all(queryClient);
  // A question may already wait: the backend runs plugins before this listener exists.
  void invalidateCommandApprovals(queryClient);

  useEventListener(pluginsEvents.pluginEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    switch (e.payload.type) {
      case 'sync':
        void pluginsCache.invalidate.all(queryClient);
        break;
      case 'edit':
        void pluginsCache.invalidate.full(queryClient, e.payload.plugin_id);
        break;
      case 'commandApprovalRequested':
        // The window may be behind another or minimised, and the plugin waits for the answer.
        getCurrentWindow()
          .requestUserAttention(UserAttentionType.Informational)
          .catch((err) => logError('requestUserAttention failed', err));
        closeCommandNotAnswered(e.payload.plugin_id);
        void invalidateCommandApprovals(queryClient);
        break;
      case 'commandApprovalResolved':
        // Only a question nobody answered is news: the user caused the other endings.
        if (e.payload.outcome === 'timedOut') {
          showCommandNotAnswered(e.payload.plugin_id, {
            title: t('plugin.commandApproval.notAnsweredTitle', {
              plugin: e.payload.plugin_id,
            }),
            description: t('plugin.commandApproval.notAnswered'),
          });
        }

        void invalidateCommandApprovals(queryClient);
        break;
    }
  });
};
