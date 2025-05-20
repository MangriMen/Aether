import { Icon } from '@iconify-icon/solid';
import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';
import { For, splitProps } from 'solid-js';

import type { TabsProps } from '@/shared/ui';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';

import type { Instance } from '@/entities/instances';

import { useTranslate } from '@/shared/model';

import {
  INSTANCE_SETTINGS_TABS_CONTENT,
  INSTANCE_SETTINGS_TABS_TRIGGER,
  InstanceSettingsDialogTabs,
} from '../model';
import { cn } from '@/shared/lib';

export type InstanceSettingsDialogBodyProps<T extends ValidComponent = 'div'> =
  TabsProps<T> & {
    instance: Instance;
  };

const InstanceSettingsDialogBody = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, InstanceSettingsDialogBodyProps<T>>,
) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslate();

  return (
    <Tabs
      class={cn('h-96 flex overflow-hidden', local.class)}
      defaultValue={InstanceSettingsDialogTabs.General}
      orientation='vertical'
      {...(others as TabsProps<T>)}
    >
      <TabsList class='min-w-40 justify-start bg-secondary-dark p-0 pl-1 pt-1'>
        <For each={INSTANCE_SETTINGS_TABS_TRIGGER}>
          {(tab) => (
            <TabsTrigger class='w-full justify-start gap-2' value={tab.value}>
              <Icon class='text-lg' icon={tab.icon} />
              {t(`instanceSettings.${tab.title}`)}
            </TabsTrigger>
          )}
        </For>
      </TabsList>
      <For each={INSTANCE_SETTINGS_TABS_CONTENT}>
        {(tabContent) => (
          <TabsContent
            value={tabContent.value}
            as={tabContent.component}
            class=' flex-1 overflow-y-auto px-1 pt-1 duration-300 animate-in slide-in-from-bottom-6 data-[orientation=vertical]:ml-8'
            instance={local.instance}
          />
        )}
      </For>
    </Tabs>
  );
};

export default InstanceSettingsDialogBody;
