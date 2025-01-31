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

import { CreateCustomInstanceDialogBody } from './CreateCustomInstanceDialogBody';
import type { DialogRootProps } from '@kobalte/core/dialog';
import { CreateInstancePluginsMenu } from './CreateInstancePluginsMenu';

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
              {t('createInstance.custom')}
            </TabsTrigger>
            <TabsTrigger value={CreateInstanceDialogTabs.Import}>
              {t('createInstance.import')}
            </TabsTrigger>
          </TabsList>
          <Separator class='mb-4 mt-2' />
          <TabsContent value={CreateInstanceDialogTabs.Custom}>
            <CreateCustomInstanceDialogBody onOpenChange={props.onOpenChange} />
          </TabsContent>
          <TabsContent value={CreateInstanceDialogTabs.Import}>
            <CreateInstancePluginsMenu onOpenChange={props.onOpenChange} />
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
};
