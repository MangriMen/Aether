import { type Component, Show, splitProps } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { usePlugins } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { PluginsList } from './PluginsList';
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
        fallback={<span>{t('plugins.noPlugins')}</span>}
        when={plugins.data?.length}
      >
        <PluginsList isLoading={plugins.isLoading} plugins={plugins.data} />
      </Show>
    </SettingsPane>
  );
};
