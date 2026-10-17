import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import { For, splitProps } from 'solid-js';

import type { TabsProps } from '@/shared/ui-kit';

import { useEditInstance, type Instance } from '@/features/instance';
import { useDefaultInstanceSettings } from '@/features/instance-defaults';
import { useTranslation } from '@/shared/i18n';
import {
  SettingsTabsContent,
  SettingsTabsList,
  SettingsTabsTrigger,
} from '@/shared/settings-layout';
import { Button, Tabs } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import {
  INSTANCE_SETTINGS_TABS_CONTENT,
  INSTANCE_SETTINGS_TABS_TRIGGER,
  InstanceSettingsDialogTabs,
} from '../model/instanceSettingsTabs';

type InstanceSettingsDialogBodyProps<T extends ValidComponent = 'div'> =
  TabsProps<T> & {
    instance: Instance;
  };

const InstanceSettingsDialogBody = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, InstanceSettingsDialogBodyProps<T>>,
) => {
  const [local, others] = splitProps(props, ['instance', 'onChange', 'class']);

  const [{ t }] = useTranslation();

  const globalSettings = useDefaultInstanceSettings();

  const editInstance = useEditInstance();

  const handleOnTabChange = (value: string) => {
    local.onChange?.(value);
  };

  return (
    <Tabs
      class={cn('h-98 p-0.5 flex overflow-hidden', local.class)}
      defaultValue={InstanceSettingsDialogTabs.General}
      orientation='vertical'
      onChange={handleOnTabChange}
      {...(others as TabsProps<T>)}
    >
      <SettingsTabsList>
        <For each={INSTANCE_SETTINGS_TABS_TRIGGER}>
          {(tab) => (
            <SettingsTabsTrigger
              value={tab.value}
              as={Button}
              class='h-auto'
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
            class='pb-6 pr-0.5'
            instance={local.instance}
            editInstance={editInstance.mutateAsync}
            defaultSettings={globalSettings.data}
          />
        )}
      </For>
    </Tabs>
  );
};

export default InstanceSettingsDialogBody;
