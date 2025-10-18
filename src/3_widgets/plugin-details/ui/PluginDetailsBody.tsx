import { splitProps, type Component } from 'solid-js';

import type { Plugin } from '@/entities/plugins';
import type { TabsProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';

import { PluginDetailsTabs } from '../model';
import { PluginDescriptionTab } from './PluginDescriptionTab';
import { PluginSettingsTab } from './PluginSettingsTab';

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

  return (
    <Tabs
      class={cn(local.class)}
      defaultValue={PluginDetailsTabs.Description}
      {...others}
    >
      <TabsList>
        <TabsTrigger value={PluginDetailsTabs.Description}>
          {t('plugin.description')}
        </TabsTrigger>
        <TabsTrigger value={PluginDetailsTabs.Settings}>
          {t('plugin.settings')}
        </TabsTrigger>
      </TabsList>
      <TabsContent
        value={PluginDetailsTabs.Description}
        as={PluginDescriptionTab}
        plugin={local.plugin}
        class='flex-1 overflow-y-auto'
      />
      <TabsContent
        value={PluginDetailsTabs.Settings}
        as={PluginSettingsTab}
        class='flex-1 overflow-y-auto'
        plugin={local.plugin}
        disabled={local.isSettingsDisabled}
      />
    </Tabs>
  );
};
