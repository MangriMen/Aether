import type { Plugin } from '@/entities/plugins';

import { disablePlugin, enablePlugin, refetchPlugin } from '@/entities/plugins';

import { cn } from '@/shared/lib';
import { Button, Separator, SettingsPane } from '@/shared/ui';
import {
  createMemo,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { PluginSettingsForm } from './PluginSettingsForm';
import { useTranslate } from '@/shared/model';

export type PluginInfoCardProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginInfoCard: Component<PluginInfoCardProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [{ t }] = useTranslate();

  const isPluginEnabled = createMemo(() => local.plugin.enabled);

  const [isLoading, setIsLoading] = createSignal(false);

  const togglePluginEnabled = async () => {
    if (isLoading()) {
      return;
    }

    setIsLoading(true);
    try {
      if (isPluginEnabled()) {
        await disablePlugin(local.plugin.metadata.plugin.id);
      } else {
        await enablePlugin(local.plugin.metadata.plugin.id);
      }
    } catch {
      /* empty */
    }
    await refetchPlugin(local.plugin.metadata.plugin.id);
    setIsLoading(false);
  };

  const isSettingsFormDisabled = createMemo(
    () => isPluginEnabled() || isLoading(),
  );

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <div>
        <div class='flex items-end gap-2'>
          <h2 class='text-xl font-bold'>{local.plugin.metadata.plugin.name}</h2>
          <span class='text-muted-foreground'>
            {t('common.version')}: {local.plugin.metadata.plugin.version}
          </span>
        </div>
        <span class='text-muted-foreground'>
          {t('common.authors')}:{' '}
          {local.plugin.metadata.plugin.authors?.join(', ')}
        </span>
      </div>
      <div>
        <Button size='sm' onClick={togglePluginEnabled} loading={isLoading()}>
          {isPluginEnabled() ? t('plugins.disable') : t('plugins.enable')}
        </Button>
      </div>

      <Separator />

      <p class='pb-1'>{local.plugin.metadata.plugin.description}</p>

      <SettingsPane
        class={cn('p-0 bg-[unset]', {
          'text-muted-foreground': isSettingsFormDisabled(),
        })}
        label={`${t('settings.title')} ${isSettingsFormDisabled() ? '(' + t('plugins.disableToChangeSettings') + ')' : ''}`}
        collapsible
      >
        <PluginSettingsForm
          plugin={local.plugin.metadata}
          disabled={isSettingsFormDisabled()}
        />
      </SettingsPane>
    </div>
  );
};
