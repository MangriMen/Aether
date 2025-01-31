import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps } from 'solid-js';
import { mergeProps, Show, splitProps } from 'solid-js';

import { Button } from './Button';
import type { DialogContentProps } from './Dialog';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
} from './Dialog';
import { useTranslate } from '@/shared/model';

export type CombinedDialogProps = DialogRootProps &
  Pick<DialogContentProps, 'variant'> & {
    header?: string;
    description?: string;
    buttonOkText?: string;
    buttonCancelText?: string;
    onOk?: ComponentProps<'button'>['onClick'];
    onCancel?: ComponentProps<'button'>['onClick'];
  };

export const CombinedDialog: Component<CombinedDialogProps> = (props) => {
  const [{ t }] = useTranslate();

  const mergedProps = mergeProps(
    {
      buttonOkText: t('common.ok') || 'Ok',
      buttonCancelText: t('common.cancel') || 'Cancel',
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
          <Button variant='destructive' onClick={local.onOk}>
            {local.buttonOkText}
          </Button>
          <Button variant='secondary' onClick={local.onCancel}>
            {local.buttonCancelText}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
