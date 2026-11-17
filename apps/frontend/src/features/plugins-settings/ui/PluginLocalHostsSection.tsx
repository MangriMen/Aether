import { For, splitProps, type Component, type ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Button, FieldLabel } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

export type PluginLocalHostsSectionProps = ComponentProps<'div'> & {
  /** `host` or `host:port`, as the settings list them. */
  localHosts: readonly string[];
  /** Takes one back: the plugin's requests there are refused again. */
  onRevoke: (localHost: string) => void;
  disabled?: boolean;
};

/**
 * The local addresses the user let a plugin reach, each with a way to take it back. One is added
 * only from the error that refused it, behind a warning, never typed in here.
 */
export const PluginLocalHostsSection: Component<
  PluginLocalHostsSectionProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'localHosts',
    'onRevoke',
    'disabled',
    'class',
  ]);
  const [{ t }] = useTranslation();

  return (
    <div class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <FieldLabel>{t('pluginSettings.localHosts.title')}</FieldLabel>
      <span class='text-sm text-muted-foreground'>
        {t('pluginSettings.localHosts.description')}
      </span>
      <ul class='gap-2 flex flex-col'>
        <For each={local.localHosts}>
          {(localHost) => (
            <li class='gap-3 flex items-center justify-between'>
              <span class='min-w-0 font-mono text-sm break-all'>
                {localHost}
              </span>
              <Button
                size='sm'
                variant='outline'
                disabled={local.disabled}
                onClick={() => local.onRevoke(localHost)}
              >
                {t('pluginSettings.localHosts.revoke')}
              </Button>
            </li>
          )}
        </For>
      </ul>
    </div>
  );
};
