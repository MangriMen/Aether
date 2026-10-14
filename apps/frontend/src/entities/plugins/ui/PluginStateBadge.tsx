import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

import type { Plugin } from '../model/plugin';

import { getTranslatedStateReason } from '../model/pluginState';

export type PluginStateBadgeProps = ComponentProps<'span'> & {
  plugin: Pick<Plugin, 'state' | 'state_reason'>;
};

/**
 * Short label for a plugin the core refused to run: `Incompatible` or
 * `Failed`. The tooltip says why.
 */
export const PluginStateBadge: Component<PluginStateBadgeProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [{ t }] = useTranslation();

  const isIncompatible = () => local.plugin.state === 'Incompatible';
  const isRefused = () => isIncompatible() || local.plugin.state === 'Failed';

  return (
    <Show when={isRefused()}>
      <CombinedTooltip
        label={getTranslatedStateReason(local.plugin.state_reason, t)}
        as='span'
        tabIndex={0}
        class={cn('text-xs font-medium text-destructive', local.class)}
        {...others}
      >
        {isIncompatible()
          ? t('plugin.stateIncompatible')
          : t('plugin.stateFailed')}
      </CombinedTooltip>
    </Show>
  );
};
