import {
  splitProps,
  type Component,
  For,
  createEffect,
  createMemo,
  createSignal,
} from 'solid-js';
import { Dynamic } from 'solid-js/web';

import type { Plugin } from '@/features/plugins';
import type { TabsProps } from '@/shared/ui-kit';

import { usePluginReadme } from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

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

  const readme = usePluginReadme(
    () => local.plugin.manifest.metadata.id,
    () => local.plugin.manifest.metadata.version,
  );

  const tabs = createMemo(() =>
    PLUGIN_DETAILS_TABS.filter((tab) => {
      switch (tab.value) {
        case PluginDetailsTabs.Description:
          return Boolean(readme.data);
        case PluginDetailsTabs.Capabilities:
          return Boolean(local.plugin.capabilities);
        default:
          return true;
      }
    }),
  );

  // The README arrives after the tabs are shown: it takes the first place unless the user has
  // already chosen a tab by then.
  const [selectedTab, setSelectedTab] = createSignal<string>(
    PluginDetailsTabs.Capabilities,
  );
  const [hasChosenTab, setHasChosenTab] = createSignal(false);

  createEffect(() => {
    if (readme.data && !hasChosenTab()) {
      setSelectedTab(PluginDetailsTabs.Description);
    }
  });

  // Not in `onChange`: the tabs report a change on their own too, when the selected one is gone.
  const markChosen = () => setHasChosenTab(true);

  return (
    <Tabs
      class={cn('flex flex-1 flex-col overflow-hidden', local.class)}
      value={selectedTab()}
      onChange={setSelectedTab}
      orientation='horizontal'
      {...others}
    >
      <TabsList
        class='self-start'
        onPointerDown={markChosen}
        onKeyDown={markChosen}
      >
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
