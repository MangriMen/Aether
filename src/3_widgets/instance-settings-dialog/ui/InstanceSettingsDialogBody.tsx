import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import { For, splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';
import type { TabsProps } from '@/shared/ui';

import { useSettings } from '@/entities/settings';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  SettingsTabsContent,
  SettingsTabsList,
  SettingsTabsTrigger,
  Tabs,
} from '@/shared/ui';

import {
  INSTANCE_SETTINGS_TABS_CONTENT,
  INSTANCE_SETTINGS_TABS_TRIGGER,
  InstanceSettingsDialogTabs,
} from '../model';

export type InstanceSettingsDialogBodyProps<T extends ValidComponent = 'div'> =
  {
    instance: Instance;
  } & TabsProps<T>;

const InstanceSettingsDialogBody = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, InstanceSettingsDialogBodyProps<T>>,
) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const settings = useSettings();

  return (
    <Tabs
      class={cn('h-96 flex overflow-hidden', local.class)}
      defaultValue={InstanceSettingsDialogTabs.General}
      orientation='vertical'
      {...(others as TabsProps<T>)}
    >
      <SettingsTabsList>
        <For each={INSTANCE_SETTINGS_TABS_TRIGGER}>
          {(tab) => (
            <SettingsTabsTrigger
              as={Button}
              leadingIcon={tab.icon}
              value={tab.value}
              variant={null}
            >
              {t(`instanceSettings.${tab.title}`)}
            </SettingsTabsTrigger>
          )}
        </For>
      </SettingsTabsList>
      <For each={INSTANCE_SETTINGS_TABS_CONTENT}>
        {(tabContent) => (
          <SettingsTabsContent
            as={tabContent.component}
            instance={local.instance}
            settings={settings.data}
            value={tabContent.value}
          />
        )}
      </For>
    </Tabs>
  );
};

export default InstanceSettingsDialogBody;
