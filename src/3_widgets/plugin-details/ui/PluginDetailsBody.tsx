import { splitProps, type Component, For, createMemo } from 'solid-js';

import type { Plugin } from '@/entities/plugins';
import type { TabsProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';

import { PLUGIN_DETAILS_TABS, PluginDetailsTabs } from '../model';

export type PluginDetailsBodyProps = TabsProps & {
  plugin: Plugin;
  isSettingsDisabled?: boolean;
};

export const PluginDetailsBody: Component<PluginDetailsBodyProps> = (props) => {
  const [local, others] = splitProps(props, [
    'plugin',
    'isSettingsDisabled',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const tabs = createMemo(() => {
    if (!local.plugin.capabilities) {
      return PLUGIN_DETAILS_TABS.filter(
        (tab) => tab.value !== PluginDetailsTabs.Capabilities,
      );
    }
    return PLUGIN_DETAILS_TABS;
  });

  return (
    <Tabs
      class={cn('flex flex-col', local.class)}
      defaultValue={PluginDetailsTabs.Description}
      orientation='horizontal'
      {...others}
    >
      <TabsList class='self-start'>
        <For each={tabs()}>
          {(tab) => (
            <TabsTrigger value={tab.value}>
              {t(`plugin.${tab.label}`)}
            </TabsTrigger>
          )}
        </For>
      </TabsList>
      <For each={tabs()}>
        {(tab) => (
          <TabsContent
            class='flex-1 overflow-hidden'
            value={tab.value}
            as={tab.component}
            plugin={local.plugin}
            isSettingsDisabled={local.isSettingsDisabled}
          />
        )}
      </For>
    </Tabs>
  );
};
