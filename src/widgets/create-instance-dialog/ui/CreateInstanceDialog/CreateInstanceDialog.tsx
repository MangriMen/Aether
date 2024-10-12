import { Component } from 'solid-js';

import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui';

import { CreateCustomInstanceDialogBody } from '../CreateCustomInstanceDialogBody';

import { CreateInstanceDialogProps } from '.';

export const CreateInstanceDialog: Component<CreateInstanceDialogProps> = (
  props,
) => {
  return (
    <Dialog {...props}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Instance</DialogTitle>
        </DialogHeader>
        <CreateCustomInstanceDialogBody onOpenChange={props.onOpenChange} />
      </DialogContent>
    </Dialog>
  );
};
