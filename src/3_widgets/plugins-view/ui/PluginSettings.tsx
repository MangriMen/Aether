import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { Plugin } from '@/entities/plugins';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { PluginSettingsForm } from './PluginSettingsForm';

export type PluginSettingsProps = ComponentProps<'div'> & {
  plugin: Plugin;
  disabled?: boolean;
};

export const PluginSettings: Component<PluginSettingsProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'disabled', 'class']);

  const [{ t }] = useTranslation();

  return (
    <SettingsPane
      class={cn('p-0 bg-[unset]', {
        'text-muted-foreground': local.disabled,
      })}
      label={`${t('settings.title')} ${local.disabled ? '(' + t('plugins.disableToChangeSettings') + ')' : ''}`}
      collapsible
      {...others}
    >
      <PluginSettingsForm
        pluginManifest={local.plugin.manifest}
        disabled={local.disabled}
      />
    </SettingsPane>
  );
};
