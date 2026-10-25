import {
  createMemo,
  createSignal,
  Match,
  Show,
  Switch,
  type Component,
} from 'solid-js';

import type { SettingsPaneProps } from '@/shared/settings-layout';

import { usePlugins } from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { SettingsPane } from '@/shared/settings-layout';
import { LoadError } from '@/shared/ui-kit';

import type { PluginSourceType } from '../model/pluginProvider';

import { pluginsPaneState } from '../model/pluginsPaneState';
import { AddPluginView } from './AddPluginView';
import { PluginDetails } from './PluginDetails';
import { PluginsPaneAddTitle } from './PluginsPaneAddTitle';
import { PluginsPaneTitle } from './PluginsPaneTitle';
import { PluginsView } from './PluginsView';

export type PluginsPaneProps = SettingsPaneProps;

type PluginPaneTab = 'view' | 'add';

export const PluginsPane: Component<PluginsPaneProps> = (props) => {
  const [{ t }] = useTranslation();

  const plugins = usePlugins();

  const [tab, setTab] = createSignal<PluginPaneTab>('view');

  const handlePluginAdd = () => {
    setTab('add');
  };

  const handleBack = () => {
    setTab('view');
  };

  const [pluginAddProvider, setPluginAddProvider] =
    createSignal<PluginSourceType | null>('git_hub');

  const paneState = createMemo(() =>
    pluginsPaneState({
      isError: plugins.isError,
      isLoading: plugins.isLoading,
      count: plugins.data?.length,
    }),
  );

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
        <Switch
          fallback={
            <PluginsView
              class='size-full'
              plugins={plugins.data}
              isLoading={plugins.isLoading}
              pluginDetails={PluginDetails}
            />
          }
        >
          <Match when={paneState() === 'error'}>
            <LoadError
              variant='inline'
              message={t('common.loadFailed')}
              retryLabel={t('common.retry')}
              onRetry={() => void plugins.refetch()}
            />
          </Match>
          <Match when={paneState() === 'empty'}>
            <span class='text-sm text-muted-foreground'>
              {t('plugins.noPlugins')}
            </span>
          </Match>
        </Switch>
      </Show>
    </SettingsPane>
  );
};
