import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { Plugin } from '../model/plugin';

import {
  getTranslatedStateLabel,
  getTranslatedStateReason,
} from '../model/pluginState';

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

  const label = () => getTranslatedStateLabel(local.plugin.state, t);

  return (
    <Show when={label()}>
      <CombinedTooltip
        label={getTranslatedStateReason(local.plugin.state_reason, t)}
        as='span'
        tabIndex={0}
        class={cn('text-xs font-medium text-destructive', local.class)}
        {...others}
      >
        {label()}
      </CombinedTooltip>
    </Show>
  );
};
