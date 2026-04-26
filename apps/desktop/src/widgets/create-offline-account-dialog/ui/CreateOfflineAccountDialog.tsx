import type { DialogRootProps } from '@kobalte/core/dialog';
import type { FormErrors } from '@modular-forms/solid';

import { createEffect, createSignal, type Component } from 'solid-js';

import type { CreateOfflineAccountFormValues } from '../model';

import {
  isAuthValidationError,
  useCreateOfflineAccount,
} from '../../../entities/account';
import {
  getTranslatedError,
  isLauncherError,
  useTranslation,
} from '../../../shared/model';
import { Dialog, DialogContent, DialogHeader } from '../../../shared/ui';
import { CreateOfflineAccountForm } from './CreateOfflineAccountForm';

export type CreateOfflineAccountDialogProps = DialogRootProps;

export const CreateOfflineAccountDialog: Component<
  CreateOfflineAccountDialogProps
> = (props) => {
  const [{ t }] = useTranslation();

  const createOfflineAccount = useCreateOfflineAccount();

  const [externalErrors, setExternalErrors] = createSignal<
    FormErrors<CreateOfflineAccountFormValues> | undefined
  >(undefined);

  const handleCreate = async (username: string) => {
    try {
      await createOfflineAccount.mutateAsync(username);
      closeDialog();
    } catch (err) {
      if (
        isLauncherError(err) &&
        err.type === 'auth' &&
        isAuthValidationError(err.payload)
      ) {
        setExternalErrors({ username: getTranslatedError(err, t) });
      }
    }
  };

  const closeDialog = () => {
    props.onOpenChange?.(false);
  };

  const resetExternalErrors = () => {
    setExternalErrors(undefined);
  };

  createEffect(() => {
    if (!props.open) {
      resetExternalErrors();
    }
  });

  return (
    <Dialog {...props}>
      <DialogContent class='max-w-80'>
        <DialogHeader>{t('createOfflineAccount.title')}</DialogHeader>
        <CreateOfflineAccountForm
          onCreate={handleCreate}
          onCancel={closeDialog}
          externalErrors={externalErrors()}
          onClearErrors={resetExternalErrors}
          isLoading={createOfflineAccount.isPending}
        />
      </DialogContent>
    </Dialog>
  );
};
