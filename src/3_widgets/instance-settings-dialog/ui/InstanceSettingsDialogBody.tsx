import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';
import { For, splitProps } from 'solid-js';

import type { TabsProps } from '@/shared/ui';
import {
  Button,
  SettingsTabsContent,
  SettingsTabsList,
  SettingsTabsTrigger,
  Tabs,
} from '@/shared/ui';

import { useEditInstance, type Instance } from '@/entities/instances';

import { useTranslation } from '@/shared/model';

import {
  INSTANCE_SETTINGS_TABS_CONTENT,
  INSTANCE_SETTINGS_TABS_TRIGGER,
  InstanceSettingsDialogTabs,
} from '../model';
import { cn } from '@/shared/lib';
import { useDefaultInstanceSettings } from '@/entities/settings';

export type InstanceSettingsDialogBodyProps<T extends ValidComponent = 'div'> =
  TabsProps<T> & {
    instance: Instance;
  };

const InstanceSettingsDialogBody = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, InstanceSettingsDialogBodyProps<T>>,
) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const globalSettings = useDefaultInstanceSettings();

  const editInstance = useEditInstance();

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
              value={tab.value}
              as={Button}
              variant={null}
              leadingIcon={tab.icon}
            >
              {t(`instanceSettings.tab.${tab.title}`)}
            </SettingsTabsTrigger>
          )}
        </For>
      </SettingsTabsList>
      <For each={INSTANCE_SETTINGS_TABS_CONTENT}>
        {(tabContent) => (
          <SettingsTabsContent
            value={tabContent.value}
            as={tabContent.component}
            instance={local.instance}
            editInstance={editInstance.mutateAsync}
            globalSettings={globalSettings.data}
          />
        )}
      </For>
    </Tabs>
  );
};

export default InstanceSettingsDialogBody;
