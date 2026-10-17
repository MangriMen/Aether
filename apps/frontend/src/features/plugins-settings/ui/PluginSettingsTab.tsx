import { splitProps, type Component, type ComponentProps } from 'solid-js';

import {
  useEditPluginSettings,
  usePluginSettings,
  type Plugin,
} from '@/features/plugins';
import { cn } from '@/shared/utils';

import { usePluginSettingsHandler } from '../lib/usePluginSettingsHandler';
import { PluginRestartNotice } from './PluginRestartNotice';
import { PluginSettingsForm } from './PluginSettingsForm';

export type PluginSettingsTabProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

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
