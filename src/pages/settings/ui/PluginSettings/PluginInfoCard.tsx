import {
  disablePlugin,
  enablePlugin,
  getIsPluginEnabled,
  type PluginMetadata,
} from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { Button, Separator } from '@/shared/ui';
import {
  createMemo,
  createResource,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { PluginSettingsForm } from './PluginSettingsForm';
import { SettingsPane } from '../SettingsPane';

export type PluginInfoCardProps = ComponentProps<'div'> & {
  plugin: PluginMetadata;
};

export const PluginInfoCard: Component<PluginInfoCardProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [isLoading, setIsLoading] = createSignal(false);

  const [isPluginEnabled, { refetch }] = createResource(() =>
    getIsPluginEnabled(local.plugin.plugin.id),
  );

  const togglePluginEnabled = async () => {
    if (isLoading()) {
      return;
    }

    setIsLoading(true);
    try {
      if (isPluginEnabled()) {
        await disablePlugin(local.plugin.plugin.id);
      } else {
        await enablePlugin(local.plugin.plugin.id);
      }
    } catch {
      /* empty */
    }
    refetch();
    setIsLoading(false);
  };

  const isSettingsFormDisabled = createMemo(
    () => isPluginEnabled() || isLoading(),
  );

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <div>
        <div class='flex items-end gap-2'>
          <h2 class='text-xl font-bold'>{local.plugin.plugin.name}</h2>
          <span class='text-muted-foreground'>
            Version: {local.plugin.plugin.version}
          </span>
        </div>
        <span class='text-muted-foreground'>
          Authors: {local.plugin.plugin.authors?.join(', ')}
        </span>
      </div>
      <div>
        <Button
          size='sm'
          onClick={togglePluginEnabled}
          loading={isPluginEnabled.loading || isLoading()}
        >
          {isPluginEnabled() ? 'Disable' : 'Enable'}
        </Button>
      </div>

      <Separator />

      <p class='pb-1'>{local.plugin.plugin.description}</p>

      <SettingsPane
        class={cn('p-0 bg-[unset]', {
          'text-muted-foreground': isSettingsFormDisabled(),
        })}
        label={`Settings ${isSettingsFormDisabled() ? '(disable plugin to change settings)' : ''}`}
      >
        <PluginSettingsForm
          plugin={local.plugin}
          disabled={isSettingsFormDisabled()}
        />
      </SettingsPane>
    </div>
  );
};
