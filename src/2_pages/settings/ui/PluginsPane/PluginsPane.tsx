import { Show, splitProps, type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { usePlugins } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';
import { PluginsView } from '@/widgets/plugins-view';

import { PluginsPaneTitle } from './PluginsPaneTitle';

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
        when={!plugins.isError || (plugins.data && plugins.data.length === 0)}
        fallback={<span>{t('plugins.noPlugins')}</span>}
      >
        <PluginsView plugins={plugins.data} isLoading={plugins.isLoading} />
      </Show>
    </SettingsPane>
  );
};
