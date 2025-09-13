import type { RouteSectionProps } from '@solidjs/router';
import {
  type ValidComponent,
  For,
  splitProps,
  type ComponentProps,
} from 'solid-js';

import type { TabsProps } from '@/shared/ui';
import {
  Button,
  SettingsTabsContent,
  SettingsTabsList,
  SettingsTabsTrigger,
  Tabs,
} from '@/shared/ui';
import { SettingsTabs } from '../model/settingsTabs';
import type { PolymorphicProps } from '@kobalte/core';
import { useTranslation } from '@/shared/model';
import { VersionInfo } from './VersionInfo';
import { useSettingsPagePrefetch, useSettingsPageTabs } from '../lib';

export type SettingsPageProps<T extends ValidComponent> = ComponentProps<T> &
  RouteSectionProps;

export const SettingsPage = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SettingsPageProps<T>>,
) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);

  const [{ t }] = useTranslation();

  const [tabsTriggers, tabsContents] = useSettingsPageTabs();

  useSettingsPagePrefetch();

  return (
    <div class='flex size-full flex-col p-4' {...others}>
      <Tabs
        class='flex size-full overflow-hidden'
        defaultValue={SettingsTabs.Appearance}
        orientation='vertical'
        {...(others as TabsProps<T>)}
      >
        <SettingsTabsList class='w-56'>
          <For each={tabsTriggers()}>
            {(tab) => (
              <SettingsTabsTrigger
                value={tab.value}
                as={Button}
                variant={null}
                leadingIcon={tab.icon}
                children={t(`settings.tab.${tab.title}`)}
              />
            )}
          </For>
          <VersionInfo class='mt-auto self-start text-sm' />
        </SettingsTabsList>
        <For each={tabsContents()}>
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
