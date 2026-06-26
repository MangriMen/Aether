import {
  createMemo,
  Show,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useUpdatesData } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { Plugin } from '../model/plugin';

export type PluginUpdateBadgeProps = ComponentProps<'span'> & {
  plugin: Plugin;
};

/**
 * Text-only badge indicating an update is available for a plugin.
 * Reads cached update data only — does NOT trigger a network request.
 */
export const PluginUpdateBadge: Component<PluginUpdateBadgeProps> = (props) => {
  const [{ t }] = useTranslation();

  const pluginId = () => props.plugin.manifest.metadata.id;
  const updates = useUpdatesData(pluginId);

  const hasUpdate = createMemo(() => updates.data?.has_update ?? false);

  return (
    <Show when={hasUpdate()}>
      <span class={cn('text-xs text-warning font-medium', props.class)}>
        {t('plugins.updateAvailable')}
      </span>
    </Show>
  );
};
