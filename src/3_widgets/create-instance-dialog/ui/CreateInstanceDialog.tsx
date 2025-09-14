import type { DialogRootProps } from '@kobalte/core/dialog';

import { type Component, For } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import { useTranslation } from '@/shared/model';
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

import type { TabContentProps, TabKey } from '../model';

import { TAB_VALUES, TABS } from '../model';
import { CreateCustomInstance } from './CreateCustomInstance';
import { ImportInstance } from './ImportInstance';

const TAB_CONTENTS: Record<TabKey, Component<TabContentProps>> = {
  [TABS.Custom]: CreateCustomInstance,
  [TABS.Import]: ImportInstance,
};

export const CreateInstanceDialog: Component<DialogRootProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Dialog {...props}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('createInstance.title')}</DialogTitle>
        </DialogHeader>
        <Tabs defaultValue={TABS.Custom}>
          <TabsList class='bg-secondary-dark px-1'>
            <For each={TAB_VALUES}>
              {(tabValue) => (
                <TabsTrigger value={tabValue}>
                  {t(`createInstance.${tabValue}`)}
                </TabsTrigger>
              )}
            </For>
          </TabsList>
          <Separator class='mb-4 mt-2' />
          <div class='animate-tab-content-wrapper flex min-h-[305px] flex-1 flex-col'>
            <For each={TAB_VALUES}>
              {(tabValue) => (
                <TabsContent
                  class='animate-tab-content flex flex-col'
                  forceMount
                  tabIndex={-1}
                  value={tabValue}
                >
                  <Dynamic
                    class='grow'
                    component={TAB_CONTENTS[tabValue]}
                    onOpenChange={props.onOpenChange}
                  />
                </TabsContent>
              )}
            </For>
          </div>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
};
