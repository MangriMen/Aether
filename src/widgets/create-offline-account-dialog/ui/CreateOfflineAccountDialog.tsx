import { DialogRootProps } from '@kobalte/core/dialog';
import { Component } from 'solid-js';

import { Dialog, DialogContent, DialogHeader } from '@/shared/ui';

import {
  createOfflineAccount,
  refetchAccountStateResource,
} from '@/entities/accounts';

import { CreateOfflineAccountForm } from '@/features/create-offline-account-form';

export type CreateOfflineAccountDialogProps = DialogRootProps;

export const CreateOfflineAccountDialog: Component<
  CreateOfflineAccountDialogProps
> = (props) => {
  const handleCreate = async (username: string) => {
    await createOfflineAccount(username);
    refetchAccountStateResource();
    props.onOpenChange?.(false);
  };

  return (
    <Dialog {...props}>
      <DialogContent>
        <DialogHeader>Create offline account</DialogHeader>
        <CreateOfflineAccountForm onCreate={handleCreate} />
      </DialogContent>
    </Dialog>
  );
};
