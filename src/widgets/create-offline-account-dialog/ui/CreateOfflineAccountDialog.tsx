import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

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
    closeDialog();
  };

  const closeDialog = () => {
    props.onOpenChange?.(false);
  };

  return (
    <Dialog {...props}>
      <DialogContent class='max-w-80'>
        <DialogHeader>Create offline account</DialogHeader>
        <CreateOfflineAccountForm
          onCreate={handleCreate}
          onCancel={closeDialog}
        />
      </DialogContent>
    </Dialog>
  );
};
