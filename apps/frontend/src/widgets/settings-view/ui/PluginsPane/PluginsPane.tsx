import { createMemo, createSignal, Show, type Component } from 'solid-js';

import type { PluginSourceTypeDto } from '@/shared/api/bindings/plugin';
import type { SettingsPaneProps } from '@/shared/ui';

import { usePlugins } from '@/entities/plugins';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';
import { PluginDetails } from '@/widgets/plugin-details';
import { PluginsView } from '@/widgets/plugins-view';

import { AddPluginView } from './AddPluginView';
import { PluginsPaneAddTitle } from './PluginsPaneAddTitle';
import { PluginsPaneTitle } from './PluginsPaneTitle';

export type PluginsPaneProps = SettingsPaneProps;

type PluginPaneTab = 'view' | 'add';

export const PluginsPane: Component<PluginsPaneProps> = (props) => {
  const [{ t }] = useTranslation();

  const plugins = usePlugins();

  const isPluginsLoadedWithoutErrors = createMemo(
    () => !plugins.isError || (plugins.data && plugins.data.length === 0),
  );

  const [tab, setTab] = createSignal<PluginPaneTab>('view');
  const handlePluginAdd = () => {
    setTab('add');
  };

  const handleBack = () => {
    setTab('view');
  };

  const [pluginAddProvider, setPluginAddProvider] =
    createSignal<PluginSourceTypeDto | null>('git_hub');

  return (
    <SettingsPane
      label={
        <Show
          when={tab() === 'view'}
          fallback={
            <PluginsPaneAddTitle
              provider={pluginAddProvider()}
              onProviderChange={setPluginAddProvider}
              onBackClick={handleBack}
            />
          }
        >
          <PluginsPaneTitle onPluginAddClick={handlePluginAdd} />
        </Show>
      }
      {...props}
    >
      <Show
        when={tab() === 'view'}
        fallback={
          <AddPluginView
            provider={pluginAddProvider()}
            onInstalled={handleBack}
          />
        }
      >
        <Show
          when={isPluginsLoadedWithoutErrors()}
          fallback={<span>{t('plugins.noPlugins')}</span>}
        >
          <PluginsView
            class='size-full'
            plugins={plugins.data}
            isLoading={plugins.isLoading}
            pluginDetails={PluginDetails}
          />
        </Show>
      </Show>
    </SettingsPane>
  );
};
