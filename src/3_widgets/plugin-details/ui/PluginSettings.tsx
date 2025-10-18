import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { Plugin } from '@/entities/plugins';

import { PluginSettingsForm } from '@/features/plugin-settings-form';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

export type PluginSettingsProps = ComponentProps<'div'> & {
  plugin: Plugin;
  disabled?: boolean;
};

export const PluginSettings: Component<PluginSettingsProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'disabled', 'class']);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn(
        {
          'text-muted-foreground': local.disabled,
        },
        local.class,
      )}
      {...others}
    >
      <Show when={local.disabled}>
        <span class='text-xl font-medium leading-10 text-accent-foreground'>
          {t('plugins.disableToChangeSettings')}
        </span>
      </Show>
      <PluginSettingsForm
        pluginManifest={local.plugin.manifest}
        disabled={local.disabled}
      />
    </div>
  );
};
