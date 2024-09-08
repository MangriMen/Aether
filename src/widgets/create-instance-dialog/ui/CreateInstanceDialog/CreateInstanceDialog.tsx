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
        <CreateCustomInstance
          footer={
            <DialogFooter>
              <Button size='sm' variant='success' type='submit'>
                Create
              </Button>
              <Button size='sm' onClick={() => props.onOpenChange?.(false)}>
                Cancel
              </Button>
            </DialogFooter>
          }
        />
      </DialogContent>
    </Dialog>
  );
};
