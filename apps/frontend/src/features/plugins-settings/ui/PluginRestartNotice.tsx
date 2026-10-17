import {
  createEffect,
  createMemo,
  createSignal,
  Show,
  type Component,
} from 'solid-js';

import { getTranslatedError, isLauncherError } from '@/features/errors';
import { useLastPluginReload, type Plugin } from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { cn } from '@/shared/utils';

import { RestartPluginButton } from './RestartPluginButton';

export type PluginRestartNoticeProps = {
  plugin: Plugin;
  class?: string;
};

/**
 * Says what has not taken effect yet — folders, the only settings fixed when a plugin starts —
 * with the restart that applies them right there. A refused restart (the plugin is busy) is
 * shown here until the next attempt, and the notice stays.
 */
export const PluginRestartNotice: Component<PluginRestartNoticeProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const pluginId = () => props.plugin.manifest.metadata.id;
  const lastReload = useLastPluginReload(pluginId);

  // A refusal from before the notice appeared is about an earlier need, not this one.
  const [shownAt, setShownAt] = createSignal(Date.now());
  const restartNeeded = createMemo(() => props.plugin.restart_needed);
  createEffect(() => {
    if (restartNeeded()) {
      setShownAt(Date.now());
    }
  });

  const refusal = () => {
    const reload = lastReload();

    if (
      reload?.status !== 'error' ||
      !reload.error ||
      reload.submittedAt < shownAt()
    ) {
      return null;
    }

    return isLauncherError(reload.error)
      ? getTranslatedError(reload.error, t)
      : t('plugin.restartFailed');
  };

  return (
    <Show when={props.plugin.restart_needed}>
      <div
        role='status'
        class={cn(
          'gap-2 rounded-md px-3 py-2 bg-secondary/secondary flex flex-col',
          props.class,
        )}
      >
        <div class='gap-4 flex items-center justify-between'>
          <span class='text-sm font-medium'>{t('plugin.restartNeeded')}</span>
          <RestartPluginButton pluginId={pluginId()} />
        </div>
        <Show when={refusal()}>
          {(text) => (
            <span role='alert' class='text-sm text-destructive'>
              {text()}
            </span>
          )}
        </Show>
      </div>
    </Show>
  );
};
