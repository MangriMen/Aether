import { createMemo, For, Show, type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { useContentProviders } from '@/entities/instances';
import { useSettings } from '@/entities/settings';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { groupProvidersByPlatform, platformsToChooseFor } from '../../lib';
import { PlatformProviderEntry } from './PlatformProviderEntry';

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
      <Show
        when={!providers.isError && !settings.isError}
        fallback={
          <span role='alert' class='text-destructive'>
            {t('settings.contentProviders.loadFailed')}
          </span>
        }
      >
        <Show
          when={!providers.isLoading && !settings.isLoading}
          fallback={
            <span role='status' class='text-muted-foreground'>
              {t('common.loading')}
            </span>
          }
        >
          <Show
            when={contested().length > 0}
            fallback={
              <span class='text-muted-foreground'>
                {t('settings.contentProviders.nothingToChoose')}
              </span>
            }
          >
            <For each={contested()}>
              {(group) => (
                <PlatformProviderEntry
                  variant='card'
                  platform={group.platform}
                  providers={group.providers}
                  selected={settings.data?.platformProviders[group.platform]}
                />
              )}
            </For>
          </Show>
        </Show>
      </Show>
    </SettingsPane>
  );
};
