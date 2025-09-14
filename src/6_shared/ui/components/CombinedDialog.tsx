import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps } from 'solid-js';

import { mergeProps, Show, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';

import type { DialogContentProps } from './Dialog';

import { Button } from './Button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
} from './Dialog';

export type CombinedDialogProps = {
  buttonCancelText?: string;
  buttonOkText?: string;
  description?: string;
  header?: string;
  onCancel?: ComponentProps<'button'>['onClick'];
  onOk?: ComponentProps<'button'>['onClick'];
} & DialogRootProps &
  Pick<DialogContentProps, 'variant'>;

export const CombinedDialog: Component<CombinedDialogProps> = (props) => {
  const [{ t }] = useTranslation();

  const mergedProps = mergeProps(
    {
      buttonCancelText: t('common.cancel') || 'Cancel',
      buttonOkText: t('common.ok') || 'Ok',
    },
    props,
  );

  const [local, others] = splitProps(mergedProps, [
    'variant',
    'header',
    'description',
    'buttonOkText',
    'buttonCancelText',
    'onOk',
    'onCancel',
  ]);

  return (
    <Dialog {...others}>
      <DialogContent class='bg-secondary-dark' variant={local.variant}>
        <Show when={local.header}>
          <DialogHeader class='text-lg'>{local.header}</DialogHeader>
        </Show>
        <Show when={local.description}>
          <DialogDescription class='text-base'>
            {local.description}
          </DialogDescription>
        </Show>
        <DialogFooter>
          <Button onClick={local.onOk} variant='destructive'>
            {local.buttonOkText}
          </Button>
          <Button onClick={local.onCancel} variant='secondary'>
            {local.buttonCancelText}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
