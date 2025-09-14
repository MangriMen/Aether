import {
  type Component,
  type ComponentProps,
  createMemo,
  createSignal,
  splitProps,
} from 'solid-js';

import {
  type Plugin,
  useDisablePlugin,
  useEnablePlugin,
} from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Separator, SettingsPane } from '@/shared/ui';

import { PluginSettingsForm } from './PluginSettingsForm';

export type PluginInfoCardProps = {
  plugin: Plugin;
} & ComponentProps<'div'>;

export const PluginInfoCard: Component<PluginInfoCardProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [{ t }] = useTranslation();

  const isPluginEnabled = createMemo(() => local.plugin.enabled);

  const [isLoading, setIsLoading] = createSignal(false);

  const { mutateAsync: enablePlugin } = useEnablePlugin();
  const { mutateAsync: disablePlugin } = useDisablePlugin();

  const togglePluginEnabled = async () => {
    if (isLoading()) {
      return;
    }

    setIsLoading(true);
    try {
      if (isPluginEnabled()) {
        await disablePlugin(local.plugin.manifest.metadata.id);
      } else {
        await enablePlugin(local.plugin.manifest.metadata.id);
      }
    } catch {
      /* empty */
    }
    setIsLoading(false);
  };

  const isSettingsFormDisabled = createMemo(
    () => isPluginEnabled() || isLoading(),
  );

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <div>
        <div class='flex items-end gap-2'>
          <h2 class='text-xl font-bold'>
            {local.plugin.manifest.metadata.name}
          </h2>
          <span class='text-muted-foreground'>
            {t('common.version')}: {local.plugin.manifest.metadata.version}
          </span>
        </div>
        <span class='text-muted-foreground'>
          {t('common.authors')}:{' '}
          {local.plugin.manifest.metadata.authors?.join(', ')}
        </span>
      </div>
      <div>
        <Button loading={isLoading()} onClick={togglePluginEnabled} size='sm'>
          {isPluginEnabled() ? t('common.disable') : t('common.enable')}
        </Button>
      </div>

      <Separator />

      <p class='pb-1'>{local.plugin.manifest.metadata.description}</p>

      <SettingsPane
        class={cn('p-0 bg-[unset]', {
          'text-muted-foreground': isSettingsFormDisabled(),
        })}
        collapsible
        label={`${t('settings.title')} ${isSettingsFormDisabled() ? '(' + t('plugins.disableToChangeSettings') + ')' : ''}`}
      >
        <PluginSettingsForm
          disabled={isSettingsFormDisabled()}
          pluginManifest={local.plugin.manifest}
        />
      </SettingsPane>
    </div>
  );
};
