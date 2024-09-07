import { Component } from 'solid-js';

import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui';

import { CreateCustomInstance } from '../CreateCustomInstance';

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
        <CreateCustomInstance />
        <DialogFooter>
          <Button size='sm' variant='success'>
            Create
          </Button>
          <Button size='sm'>Cancel</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
