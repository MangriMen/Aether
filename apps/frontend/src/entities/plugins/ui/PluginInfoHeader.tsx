import { Show, type Component } from 'solid-js';

import {
  PluginApiCompatibilityBadge,
  PluginSourceBadge,
} from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { Plugin } from '../model/plugin';

import {
  getTranslatedStateLabel,
  getTranslatedStateReason,
} from '../model/pluginState';

export type PluginInfoHeaderProps = {
  name: string;
  version: string;
  apiVersion?: string | null;
  /** Omit when verification does not apply (e.g. a not yet installed plugin). */
  verified?: boolean;
  authors?: string | null;
  description?: string | null;
  /** Installed plugin whose refusal reason (`Incompatible`/`Failed`) is shown. */
  plugin?: Pick<Plugin, 'state' | 'state_reason'>;
  class?: string;
};

/**
 * Reusable header info block showing plugin name, version, API badge,
 * authors, and description. Meant to be placed inside a parent flex row
 * alongside an image/icon.
 */
export const PluginInfoHeader: Component<PluginInfoHeaderProps> = (props) => {
  const [{ t }] = useTranslation();

  const refusal = () => {
    const title =
      props.plugin && getTranslatedStateLabel(props.plugin.state, t);

    if (!props.plugin || !title) {
      return null;
    }

    return {
      title,
      reason: getTranslatedStateReason(props.plugin.state_reason, t),
    };
  };

  return (
    <div class={cn('flex flex-col', props.class)}>
      {/* Title + version + badge */}
      <div class='gap-2 flex items-end'>
        <h2 class='text-xl font-bold'>{props.name}</h2>
        <div class='gap-1 flex items-end'>
          <span class='text-muted-foreground'>{props.version}</span>
          <Show when={props.apiVersion}>
            <PluginApiCompatibilityBadge
              class='mb-0.75'
              apiVersion={props.apiVersion!}
            />
          </Show>
          <Show when={props.verified !== undefined}>
            <PluginSourceBadge class='mb-0.75' verified={props.verified!} />
          </Show>
        </div>
      </div>
      {/* Why the core refused to run the plugin */}
      <Show when={refusal()}>
        {(r) => (
          <span class='text-sm text-destructive wrap-break-word'>
            {r().title}: {r().reason}
          </span>
        )}
      </Show>
      {/* Authors */}
      <Show when={props.authors}>
        <div class='gap-4 flex'>
          <span>{props.authors}</span>
        </div>
      </Show>
      {/* Description */}
      <Show when={props.description}>
        <span class='my-1'>{props.description}</span>
      </Show>
    </div>
  );
};
