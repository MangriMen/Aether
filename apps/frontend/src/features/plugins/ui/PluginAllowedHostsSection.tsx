import {
  For,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Badge, FieldLabel } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { isWideHostPattern } from '../model/wideHostPattern';

export type PluginAllowedHostsSectionProps = ComponentProps<'div'> & {
  allowedHosts: string[];
};

/** The sites a plugin asks to reach, with a warning on an entry that admits almost any site. */
export const PluginAllowedHostsSection: Component<
  PluginAllowedHostsSectionProps
> = (props) => {
  const [local, others] = splitProps(props, ['allowedHosts', 'class']);
  const [{ t }] = useTranslation();

  const hasWideEntry = () => local.allowedHosts.some(isWideHostPattern);

  return (
    <div class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <FieldLabel>{t('plugin.allowedHosts')}</FieldLabel>
      <span class='text-sm text-muted-foreground'>
        {t('plugin.allowedHostsDescription')}
      </span>
      <Show
        when={local.allowedHosts.length > 0}
        fallback={<span class='text-sm'>{t('plugin.allowedHostsNone')}</span>}
      >
        <div class='gap-1.5 flex flex-wrap'>
          <For each={local.allowedHosts}>
            {(host) => (
              <Badge variant='outline' class='font-mono'>
                {host}
              </Badge>
            )}
          </For>
        </div>
      </Show>
      <Show when={hasWideEntry()}>
        <span class='text-sm text-warning'>{t('plugin.allowedHostsWide')}</span>
      </Show>
    </div>
  );
};
