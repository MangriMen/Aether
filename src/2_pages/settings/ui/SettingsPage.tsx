import type { PolymorphicProps } from '@kobalte/core';

import { useNavigate, type RouteSectionProps } from '@solidjs/router';
import {
  type ValidComponent,
  For,
  splitProps,
  type ComponentProps,
  Show,
  createMemo,
} from 'solid-js';

import type { TabsProps } from '@/shared/ui';

import { useCheckUpdate } from '@/entities/updates';
import { checkIsUpdateAvailable } from '@/entities/updates/model';
import { ROUTES } from '@/shared/config';
import { useTranslation } from '@/shared/model';
import {
  Badge,
  Button,
  SettingsTabsContent,
  SettingsTabsList,
  SettingsTabsTrigger,
  Tabs,
} from '@/shared/ui';

import { useOnMountSettingsPage, useSettingsPageTabs } from '../lib';
import { isSettingsTab, SettingsTab } from '../model/settingsTabs';
import { VersionInfo } from './VersionInfo';

export type SettingsPageProps<T extends ValidComponent> = ComponentProps<T> &
  RouteSectionProps;

const defaultTab = SettingsTab.Appearance;

export const SettingsPage = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SettingsPageProps<T>>,
) => {
  const [local, others] = splitProps(props, ['params', 'location', 'data']);

  const navigate = useNavigate();

  const currentTab = createMemo(() => {
    const tab = local.params.tab;

    if (!tab) {
      return defaultTab;
    }

    const decoded = decodeURIComponent(tab);

    return isSettingsTab(decoded) ? decoded : defaultTab;
  });

  const handleChangeTab = (tab: string) => {
    if (isSettingsTab(tab)) {
      navigate(ROUTES.SETTINGS(tab));
    } else {
      navigate(ROUTES.SETTINGS(defaultTab));
    }
  };

  const [{ t }] = useTranslation();

  const [tabsTriggers, tabsContents] = useSettingsPageTabs();

  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  useOnMountSettingsPage();

  return (
    <div class='p-page flex size-full flex-col' {...others}>
      <Tabs
        class='flex size-full overflow-hidden'
        value={currentTab()}
        onChange={handleChangeTab}
        defaultValue={defaultTab}
        orientation='vertical'
        {...(others as TabsProps<T>)}
      >
        <SettingsTabsList class='w-56'>
          <For each={tabsTriggers()}>
            {(tab) => (
              <SettingsTabsTrigger
                value={tab.value}
                as={Button}
                class='h-auto'
                variant={null}
                leadingIcon={tab.icon}
                children={
                  <span class='overflow-hidden text-ellipsis'>
                    <Show
                      when={
                        tab.value === SettingsTab.Update && isUpdateAvailable()
                      }
                    >
                      <Badge class='absolute inset-y-0 right-2 my-auto aspect-square size-2 p-0' />
                    </Show>
                    {t(`settings.tab.${tab.title}`)}
                  </span>
                }
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
