import { usePlugins } from '@/entities/plugins';
import { Show, splitProps, type Component } from 'solid-js';
import { useTranslation } from '@/shared/model';
import { PluginsList } from './PluginsList';
import { PluginsPaneTitle } from './PluginsPaneTitle';
import type { SettingsPaneProps } from '@/shared/ui';
import { SettingsPane } from '@/shared/ui';
import { cn } from '@/shared/lib';

export type PluginsPaneProps = SettingsPaneProps;

export const PluginsPane: Component<PluginsPaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const plugins = usePlugins();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={<PluginsPaneTitle />}
      {...others}
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
