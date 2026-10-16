import { splitProps, type Component, For, createMemo } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import type { Plugin } from '@/features/plugins';
import type { TabsProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';

import {
  PLUGIN_DETAILS_TABS,
  PluginDetailsTabs,
} from '../model/pluginDetailsTabs';

export type PluginDetailsBodyProps = TabsProps & {
  plugin: Plugin;
};

export const PluginDetailsBody: Component<PluginDetailsBodyProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

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
      class={cn('flex flex-1 flex-col overflow-hidden', local.class)}
      defaultValue={PluginDetailsTabs.Capabilities}
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
            class='pl-1 flex flex-col overflow-hidden'
            value={tab.value}
          >
            <Dynamic component={tab.component} plugin={local.plugin} />
          </TabsContent>
        )}
      </For>
    </Tabs>
  );
};
