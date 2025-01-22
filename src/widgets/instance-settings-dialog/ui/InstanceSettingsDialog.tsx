import { Icon } from '@iconify-icon/solid';
import { DialogRootProps } from '@kobalte/core/dialog';
import { Component, For, splitProps } from 'solid-js';

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/shared/ui';

import { Instance } from '@/entities/instance';

import {
  INSTANCE_SETTINGS_TABS_CONTENT,
  INSTANCE_SETTINGS_TABS_TRIGGER,
  InstanceSettingsDialogTabs,
} from '../model';

export type InstanceSettingsDialogProps = DialogRootProps & {
  instance: Instance;
};

const InstanceSettingsDialog: Component<InstanceSettingsDialogProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['instance']);

  return (
    <Dialog defaultOpen open={true} {...others}>
      <DialogContent class='w-[900px] max-w-full bg-secondary-dark'>
        <DialogHeader>
          <DialogTitle>{local.instance.name}</DialogTitle>
        </DialogHeader>
        <Tabs
          class='min-h-96'
          defaultValue={InstanceSettingsDialogTabs.General}
          orientation='vertical'
        >
          <TabsList class='min-w-40 justify-start bg-secondary-dark p-0'>
            <For each={INSTANCE_SETTINGS_TABS_TRIGGER}>
              {(tab) => (
                <TabsTrigger
                  class='w-full justify-start gap-2'
                  value={tab.value}
                >
                  <Icon class='text-lg' icon={tab.icon} />
                  {tab.title}
                </TabsTrigger>
              )}
            </For>
          </TabsList>
          <For each={INSTANCE_SETTINGS_TABS_CONTENT}>
            {(tabContent) => (
              <TabsContent
                class='data-[orientation=vertical]:ml-8'
                value={tabContent.value}
              >
                <tabContent.component instance={local.instance} />
              </TabsContent>
            )}
          </For>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
};

export default InstanceSettingsDialog;
