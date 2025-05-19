import type { Component } from 'solid-js';

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

import { useTranslate } from '@/shared/model';

import { CreateCustomInstance } from './CreateCustomInstance';
import type { DialogRootProps } from '@kobalte/core/dialog';
import { ImportInstance } from './ImportInstance';

enum CreateInstanceDialogTabs {
  Custom = 'custom',
  Import = 'import',
}

export type CreateInstanceDialogProps = DialogRootProps;

export const CreateInstanceDialog: Component<CreateInstanceDialogProps> = (
  props,
) => {
  const [{ t }] = useTranslate();

  return (
    <Dialog {...props}>
      <DialogContent class='bg-secondary-dark'>
        <DialogHeader>
          <DialogTitle>{t('createInstance.title')}</DialogTitle>
        </DialogHeader>
        <Tabs defaultValue={CreateInstanceDialogTabs.Custom}>
          <TabsList class='bg-secondary-dark p-0'>
            <TabsTrigger value={CreateInstanceDialogTabs.Custom}>
              {t(`createInstance.${CreateInstanceDialogTabs.Custom}`)}
            </TabsTrigger>
            <TabsTrigger value={CreateInstanceDialogTabs.Import}>
              {t(`createInstance.${CreateInstanceDialogTabs.Import}`)}
            </TabsTrigger>
          </TabsList>
          <Separator class='mb-4 mt-2' />
          <TabsContent
            class='min-h-[294px]'
            value={CreateInstanceDialogTabs.Custom}
          >
            <CreateCustomInstance onOpenChange={props.onOpenChange} />
          </TabsContent>
          <TabsContent
            class='flex min-h-[294px] grow flex-col'
            value={CreateInstanceDialogTabs.Import}
          >
            <ImportInstance class='grow' onOpenChange={props.onOpenChange} />
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
};
