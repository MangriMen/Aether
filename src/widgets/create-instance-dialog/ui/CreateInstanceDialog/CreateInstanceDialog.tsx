import { Component } from 'solid-js';

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

import { CreateInstancePluginsMenu } from '@/features/create-instance-plugins-menu';

import { CreateCustomInstanceDialogBody } from '../CreateCustomInstanceDialogBody';

import { CreateInstanceDialogProps } from '.';

enum CreateInstanceDialogTabs {
  Custom = 'custom',
  Import = 'import',
  Plugins = 'Plugins',
}

export const CreateInstanceDialog: Component<CreateInstanceDialogProps> = (
  props,
) => {
  return (
    <Dialog {...props}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Instance</DialogTitle>
        </DialogHeader>
        <Tabs defaultValue={CreateInstanceDialogTabs.Custom}>
          <TabsList>
            <TabsTrigger value={CreateInstanceDialogTabs.Custom}>
              Custom
            </TabsTrigger>
            <TabsTrigger value={CreateInstanceDialogTabs.Plugins}>
              Plugins
            </TabsTrigger>
          </TabsList>
          <hr class='my-2' />
          <TabsContent value={CreateInstanceDialogTabs.Custom}>
            <CreateCustomInstanceDialogBody onOpenChange={props.onOpenChange} />
          </TabsContent>
          <TabsContent value={CreateInstanceDialogTabs.Plugins}>
            <CreateInstancePluginsMenu onOpenChange={props.onOpenChange} />
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
};
