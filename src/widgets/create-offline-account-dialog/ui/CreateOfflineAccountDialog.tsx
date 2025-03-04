import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

import { Dialog, DialogContent, DialogHeader } from '@/shared/ui';

import { createOfflineAccount, refetchAccounts } from '@/entities/accounts';
import { CreateOfflineAccountForm } from './CreateOfflineAccountForm';
import { useTranslate } from '@/shared/model';

export type CreateOfflineAccountDialogProps = DialogRootProps;

export const CreateOfflineAccountDialog: Component<
  CreateOfflineAccountDialogProps
> = (props) => {
  const [{ t }] = useTranslate();

  const handleCreate = async (username: string) => {
    await createOfflineAccount(username);
    refetchAccounts();
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
