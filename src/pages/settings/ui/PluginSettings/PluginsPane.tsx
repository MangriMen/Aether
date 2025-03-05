import { usePlugins } from '@/entities/plugins';
import { Show, type Component } from 'solid-js';
import type { SettingsPaneProps } from '../SettingsPane';
import { SettingsPane } from '../SettingsPane';
import { useTranslate } from '@/shared/model';
import { PluginsList } from './PluginsList';
import { PluginsPaneTitle } from './PluginsPaneTitle';

export type PluginsEntryProps = SettingsPaneProps;

export const PluginsPane: Component<PluginsEntryProps> = (props) => {
  const [{ t }] = useTranslate();

  const plugins = usePlugins();

  return (
    <SettingsPane
      class='container max-w-screen-lg'
      label={<PluginsPaneTitle />}
      {...props}
    >
      <Show
        when={plugins?.size}
        fallback={<span>{t('plugins.noPlugins')}</span>}
      >
        <PluginsList plugins={plugins} />
      </Show>
    </SettingsPane>
  );
};
