import { createMemo, Match, Switch, type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { useContentProviders } from '@/entities/instances';
import { useSettings } from '@/entities/settings';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { groupProvidersByPlatform, platformsToChooseFor } from '../../lib';
import { ContentProvidersView } from './ContentProvidersView';

export type ContentProvidersPaneProps = SettingsPaneProps;

export const ContentProvidersPane: Component<ContentProvidersPaneProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const providers = useContentProviders();
  const settings = useSettings();

  // A platform with a single provider has nothing to choose from — unless a
  // choice is stored for it, which has to stay visible to be changed.
  const contested = createMemo(() =>
    platformsToChooseFor(
      groupProvidersByPlatform(providers.data ?? []),
      settings.data?.platformProviders,
    ),
  );

  return (
    <SettingsPane label={t('settings.tab.contentProviders')} {...props}>
      <Switch>
        <Match when={providers.isError || settings.isError}>
          <span role='alert' class='text-destructive'>
            {t('settings.contentProviders.loadFailed')}
          </span>
        </Match>
        <Match when={providers.isLoading || settings.isLoading}>
          <span role='status' class='text-muted-foreground'>
            {t('common.loading')}
          </span>
        </Match>
        <Match when>
          <ContentProvidersView
            groups={contested()}
            selected={settings.data?.platformProviders}
          />
        </Match>
      </Switch>
    </SettingsPane>
  );
};
