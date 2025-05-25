import { usePlugins } from '@/entities/plugins';
import { Show, type Component } from 'solid-js';
import { useTranslate } from '@/shared/model';
import { PluginsList } from './PluginsList';
import { PluginsPaneTitle } from './PluginsPaneTitle';
import type { SettingsPaneProps } from '@/shared/ui';
import { SettingsPane } from '@/shared/ui';

export type PluginsPaneProps = SettingsPaneProps;

export const PluginsPane: Component<PluginsPaneProps> = (props) => {
  const [{ t }] = useTranslate();

  const plugins = usePlugins();

  return (
    <SettingsPane
      class='container max-w-screen-lg'
      label={<PluginsPaneTitle />}
      {...props}
    >
      <Show
        when={plugins.data?.length}
        fallback={<span>{t('plugins.noPlugins')}</span>}
      >
        <PluginsList plugins={plugins.data} isLoading={plugins.isLoading} />
      </Show>
    </SettingsPane>
  );
};
