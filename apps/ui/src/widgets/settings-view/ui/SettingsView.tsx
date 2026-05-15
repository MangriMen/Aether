import type { PolymorphicProps } from '@kobalte/core';

import {
  type ValidComponent,
  For,
  splitProps,
  type ComponentProps,
  createMemo,
} from 'solid-js';

import type { TabsProps } from '@/shared/ui';

import { UpdateBadge } from '@/features/update-badge';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  SettingsTabsContent,
  SettingsTabsList,
  SettingsTabsTrigger,
  Tabs,
} from '@/shared/ui';

import { useOnMountSettingsPage, useSettingsPageTabs } from '../lib';
import { DEFAULT_TAB, isSettingsTab, SettingsTab } from '../model';
import { VersionInfo } from './VersionInfo';

export type SettingsViewProps<T extends ValidComponent> = ComponentProps<T> & {
  tab?: SettingsTab;
  isTabListOpen?: boolean;
  onChangeTab?: (tab: SettingsTab) => void;
  onTabListOpenChange?: (open: boolean) => void;
  dialogContentRef?: (el: HTMLDivElement) => void;
};

export const SettingsView = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SettingsViewProps<T>>,
) => {
  const [local, others] = splitProps(props, [
    'tab',
    'onChangeTab',
    'isTabListOpen',
    'onTabListOpenChange',
    'dialogContentRef',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const currentTab = createMemo(() => local.tab ?? DEFAULT_TAB);

  const handleChangeTab = (tab: string) => {
    local.onChangeTab?.(isSettingsTab(tab) ? tab : DEFAULT_TAB);
  };

  const tabs = useSettingsPageTabs();

  useOnMountSettingsPage();

  return (
    <div class={cn('flex flex-col overflow-hidden', local.class)} {...others}>
      <Tabs
        class='flex grow overflow-hidden'
        value={currentTab()}
        onChange={handleChangeTab}
        defaultValue={DEFAULT_TAB}
        orientation='vertical'
        {...(others as TabsProps<T>)}
      >
        {/* <Dialog
          open={local.isTabListOpen}
          onOpenChange={local.onTabListOpenChange}
        >
          <DialogContent
            class='z-50 h-full bg-background pt-20 data-[expanded]:animate-in data-[closed]:animate-out data-[closed]:slide-out-to-left-full data-[expanded]:slide-in-from-left-full'
            variant='unstyled'
            showActions={false}
          > */}
        <SettingsTabsList class='w-56 pb-6'>
          <For each={tabs()}>
            {(tab) => (
              <SettingsTabsTrigger
                value={tab.value}
                as={Button}
                class='h-auto'
                variant={null}
                leadingIcon={tab.icon}
                children={
                  <span class='overflow-hidden text-ellipsis'>
                    <UpdateBadge
                      class='absolute inset-y-0 right-2 my-auto'
                      isAbleToShow={tab.value === SettingsTab.Update}
                    />
                    {t(`settings.tab.${tab.label}`)}
                  </span>
                }
              />
            )}
          </For>

          <VersionInfo class='mt-auto self-start pl-6 text-sm' />
        </SettingsTabsList>
        {/* </DialogContent>
        </Dialog> */}

        <For each={tabs()}>
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
