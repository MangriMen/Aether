import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

import { useCreateOfflineAccount } from '@/entities/accounts';
import { useTranslation } from '@/shared/model';
import { Dialog, DialogContent, DialogHeader } from '@/shared/ui';

import { CreateOfflineAccountForm } from './CreateOfflineAccountForm';

export type CreateOfflineAccountDialogProps = DialogRootProps;

export const CreateOfflineAccountDialog: Component<
  CreateOfflineAccountDialogProps
> = (props) => {
  const [{ t }] = useTranslation();

  const { mutateAsync: createOfflineAccount } = useCreateOfflineAccount();

  const handleCreate = async (username: string) => {
    await createOfflineAccount(username);
    closeDialog();
  };

  const closeDialog = () => {
    props.onOpenChange?.(false);
  };

  return (
    <Dialog {...props}>
      <DialogContent class='max-w-80'>
        <DialogHeader>{t('createOfflineAccount.title')}</DialogHeader>
        <CreateOfflineAccountForm
          onCreate={handleCreate}
          onCancel={closeDialog}
        />
      </DialogContent>
    </Dialog>
  );
};
