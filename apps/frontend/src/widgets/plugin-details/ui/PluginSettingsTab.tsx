import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { useEditPluginSettings, usePluginSettings } from '@/entities/plugins';
import { PluginSettingsForm } from '@/features/plugin-settings-form';
import { PluginRestartNotice } from '@/features/restart-plugin-button';
import { cn } from '@/shared/lib';

import type { PluginDetailsTabProps } from '../model';

import { usePluginSettingsHandler } from '../lib';

export type PluginSettingsTabProps = ComponentProps<'div'> &
  PluginDetailsTabProps;

export const PluginSettingsTab: Component<PluginSettingsTabProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const pluginSettings = usePluginSettings(
    () => local.plugin.manifest.metadata.id,
  );
  const editPluginSettings = useEditPluginSettings();

  const { initialValues, onChangePartial } = usePluginSettingsHandler({
    pluginId: () => local.plugin.manifest.metadata.id,
    pluginSettings: () => pluginSettings.data,
    editPluginSettings: () => editPluginSettings.mutateAsync,
  });

  return (
    <div
      class={cn('gap-2 flex grow flex-col overflow-y-auto', local.class)}
      {...others}
    >
      <PluginRestartNotice plugin={local.plugin} />
      <PluginSettingsForm
        runtimeConfig={local.plugin.manifest.runtime}
        initialValues={initialValues}
        onChangePartial={onChangePartial}
      />
    </div>
  );
};
