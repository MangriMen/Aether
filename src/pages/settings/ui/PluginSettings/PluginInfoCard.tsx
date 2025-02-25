import {
  disablePlugin,
  enablePlugin,
  getIsPluginEnabled,
  type PluginMetadata,
} from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { Button, Separator } from '@/shared/ui';
import {
  createResource,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { PluginSettingsForm } from './PluginSettingsForm';

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

      <span class='text-lg font-medium'>Settings</span>

      <PluginSettingsForm plugin={local.plugin} />
    </div>
  );
};
