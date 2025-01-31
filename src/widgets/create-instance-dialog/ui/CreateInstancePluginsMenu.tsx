import type { Component, ComponentProps } from 'solid-js';
import { createMemo, For, Show, splitProps } from 'solid-js';
import { Dynamic } from 'solid-js/web';

// TODO: move to own package
import { PackwizPluginImportMenu } from '@/plugins/packwiz-plugin';

import type { Option, Plugin } from '@/shared/model';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';

import type { DialogRootProps } from '@kobalte/core/dialog';
import { callPlugin } from '../api';

export type CreateInstancePluginsMenuProps = ComponentProps<'div'> &
  Pick<DialogRootProps, 'onOpenChange'>;

export const CreateInstancePluginsMenu: Component<
  CreateInstancePluginsMenuProps
> = (props) => {
  const [local, others] = splitProps(props, ['onOpenChange']);

  const plugins = createMemo<Plugin[]>(() => [
    {
      name: 'Packwiz',
      version: '1.0.0',
      id: 'packwiz',
      description: 'Packwiz plugin',
      component: PackwizPluginImportMenu,
    },
  ]);

  const tabs = createMemo(() =>
    plugins().map<Option>((plugin) => ({
      name: plugin.name,
      value: plugin.id,
    })),
  );

  const tabsContent = createMemo(() =>
    plugins().reduce<Record<string, Plugin['component']>>(
      (acc, plugin) => ({ ...acc, [plugin.id]: plugin.component }),
      {},
    ),
  );

  return (
    <div {...others}>
      <Tabs defaultValue={tabs()[0].value} orientation='vertical'>
        <TabsList class='bg-secondary-dark p-0'>
          <For each={tabs()}>
            {(tab) => (
              <TabsTrigger
                class='w-full min-w-24 max-w-24 justify-start'
                value={tab.value}
              >
                <span class='overflow-x-hidden text-ellipsis' title={tab.name}>
                  {tab.name}
                </span>
              </TabsTrigger>
            )}
          </For>
        </TabsList>
        <For each={tabs()}>
          {(tab) => (
            <TabsContent
              class='h-[294px] data-[orientation=vertical]:ml-6'
              value={tab.value}
            >
              <Show when={tabsContent()[tab.value]}>
                {(component) => (
                  <Dynamic
                    component={component()}
                    callPlugin={callPlugin}
                    onSubmit={() => local.onOpenChange?.(false)}
                  />
                )}
              </Show>
            </TabsContent>
          )}
        </For>
      </Tabs>
    </div>
  );
};
