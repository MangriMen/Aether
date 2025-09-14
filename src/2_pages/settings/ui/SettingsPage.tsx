import type { RouteSectionProps } from '@solidjs/router';
import {
  type ValidComponent,
  For,
  splitProps,
  type ComponentProps,
  createMemo,
} from 'solid-js';

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
import { isDeveloperMode, useTranslation } from '@/shared/model';
import { VersionInfo } from './VersionInfo';

export type SettingsPageProps<T extends ValidComponent> = ComponentProps<T> &
  RouteSectionProps;

export const SettingsPage = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SettingsPageProps<T>>,
) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);

  const [{ t }] = useTranslation();

  const tabs_triggers = createMemo(() => {
    if (!isDeveloperMode()) {
      return SETTINGS_TABS_TRIGGER.filter(
        (trigger) => trigger.value !== SettingsTabs.Experimental,
      );
    }

    return SETTINGS_TABS_TRIGGER;
  });

  const tabs_contents = createMemo(() => {
    if (!isDeveloperMode()) {
      return SETTINGS_TABS_CONTENT.filter(
        (trigger) => trigger.value !== SettingsTabs.Experimental,
      );
    }

    return SETTINGS_TABS_CONTENT;
  });

  return (
    <div class='flex size-full flex-col p-4' {...others}>
      <Tabs
        class='flex size-full overflow-hidden'
        defaultValue={SettingsTabs.Appearance}
        orientation='vertical'
        {...(others as TabsProps<T>)}
      >
        <SettingsTabsList class='w-56'>
          <For each={tabs_triggers()}>
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
          <VersionInfo class='mt-auto self-start text-sm' />
        </SettingsTabsList>
        <For each={tabs_contents()}>
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
