import type { ComponentProps } from 'solid-js';

import IconMdiAlertOutline from '~icons/mdi/alert-outline';
import { createMemo, Show, splitProps, type Component } from 'solid-js';

import { checkIsApiCompatible, useApiVersion } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

export type PluginApiCompatibilityBadgeProps = {
  /** SemVer requirement from the plugin manifest, e.g. `"^2.3.0"` */
  apiVersion: string;
};

export const PluginApiCompatibilityBadge: Component<
  ComponentProps<'span'> & PluginApiCompatibilityBadgeProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const launcherApiVersion = useApiVersion();

  const isApiCompatible = createMemo(() => {
    if (!launcherApiVersion.data) {
      return true;
    }

    return checkIsApiCompatible(launcherApiVersion.data, props.apiVersion);
  });

  return (
    <Show when={!isApiCompatible()}>
      <CombinedTooltip
        label={t('plugin.incompatibleApiVersion')}
        as='div'
        class={cn('flex text-warning', local.class)}
        {...others}
      >
        <IconMdiAlertOutline />
      </CombinedTooltip>
    </Show>
  );
};
