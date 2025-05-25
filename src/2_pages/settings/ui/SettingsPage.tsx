import type { RouteSectionProps } from '@solidjs/router';
import {
  type ValidComponent,
  For,
  splitProps,
  type ComponentProps,
} from 'solid-js';

import { AppVersion } from './AppVersion';
import type { TabsProps } from '@/shared/ui';
import {
  Button,
  SettingsTabsContent,
  SettingsTabsList,
  SettingsTabsTrigger,
  Tabs,
} from '@/shared/ui';
import {
  SETTINGS_TABS_CONTENT,
  SETTINGS_TABS_TRIGGER,
  SettingsTabs,
} from '../model/settingsTabs';
import type { PolymorphicProps } from '@kobalte/core';
import { useTranslate } from '@/shared/model';

export type SettingsPageProps<T extends ValidComponent> = ComponentProps<T> &
  RouteSectionProps;

export const SettingsPage = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SettingsPageProps<T>>,
) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);

  const [{ t }] = useTranslate();

  return (
    <div class='flex size-full flex-col bg-secondary-dark p-4' {...others}>
      <Tabs
        class='flex size-full overflow-hidden'
        defaultValue={SettingsTabs.Appearance}
        orientation='vertical'
        {...(others as TabsProps<T>)}
      >
        <SettingsTabsList class='w-56'>
          <For each={SETTINGS_TABS_TRIGGER}>
            {(tab) => (
              <SettingsTabsTrigger
                value={tab.value}
                as={Button}
                variant={null}
                leadingIcon={tab.icon}
              >
                {t(`settings.${tab.title}`)}
              </SettingsTabsTrigger>
            )}
          </For>
          <AppVersion class='mt-auto self-start text-sm' />
        </SettingsTabsList>
        <For each={SETTINGS_TABS_CONTENT}>
          {(tabContent) => (
            <SettingsTabsContent
              value={tabContent.value}
              as={tabContent.component}
            />
          )}
        </For>
      </Tabs>
    </div>
  );
};
