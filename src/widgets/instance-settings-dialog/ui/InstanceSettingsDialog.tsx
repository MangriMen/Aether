import { Icon } from '@iconify-icon/solid';
import { DialogRootProps } from '@kobalte/core/dialog';
import { Component, For, splitProps } from 'solid-js';

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  Separator,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/shared/ui';

import { Instance } from '@/entities/minecraft';

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
      <DialogContent class='w-[900px] max-w-full'>
        <DialogHeader>
          <DialogTitle>{local.instance.name}</DialogTitle>
        </DialogHeader>
        <Tabs
          class='min-h-96'
          defaultValue={InstanceSettingsDialogTabs.General}
          orientation='vertical'
        >
          <TabsList class='min-w-40 justify-start bg-background pr-3'>
            <For each={INSTANCE_SETTINGS_TABS_TRIGGER}>
              {(tab) => (
                <TabsTrigger
                  class='w-full justify-start gap-2 data-[selected]:bg-muted'
                  value={tab.value}
                >
                  <Icon class='text-lg' icon={tab.icon} />
                  {tab.title}
                </TabsTrigger>
              )}
            </For>
          </TabsList>
          <Separator orientation='vertical' />
          <For each={INSTANCE_SETTINGS_TABS_CONTENT}>
            {(tabContent) => (
              <TabsContent class='pl-2' value={tabContent.value}>
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
