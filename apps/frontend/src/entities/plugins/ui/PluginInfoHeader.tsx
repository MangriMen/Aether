import { Show, type Component } from 'solid-js';

import { PluginApiCompatibilityBadge } from '@/entities/plugins';
import { cn } from '@/shared/lib';

export type PluginInfoHeaderProps = {
  name: string;
  version: string;
  apiVersion?: string | null;
  authors?: string | null;
  description?: string | null;
  class?: string;
};

/**
 * Reusable header info block showing plugin name, version, API badge,
 * authors, and description. Meant to be placed inside a parent flex row
 * alongside an image/icon.
 */
export const PluginInfoHeader: Component<PluginInfoHeaderProps> = (props) => {
  return (
    <div class={cn('flex flex-col', props.class)}>
      {/* Title + version + badge */}
      <div class='flex items-end gap-2'>
        <h2 class='text-xl font-bold'>{props.name}</h2>
        <div class='flex items-end gap-1'>
          <span class='text-muted-foreground'>{props.version}</span>
          <Show when={props.apiVersion}>
            <PluginApiCompatibilityBadge
              class='mb-0.75'
              apiVersion={props.apiVersion!}
            />
          </Show>
        </div>
      </div>
      {/* Authors */}
      <Show when={props.authors}>
        <div class='flex gap-4'>
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
