import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps, JSX } from 'solid-js';

import { mergeProps, Show, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';

import type { ButtonProps } from './Button';
import type { DialogContentProps } from './Dialog';

import { Button } from './Button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
} from './Dialog';

export type CombinedDialogProps = DialogRootProps &
  Pick<DialogContentProps, 'variant'> & {
    header?: JSX.Element;
    description?: JSX.Element;
    buttonOkText?: string;
    buttonCancelText?: string;
    buttonOkVariant?: ButtonProps['variant'];
    onOk?: ComponentProps<'button'>['onClick'];
    onCancel?: ComponentProps<'button'>['onClick'];
  };

export const CombinedDialog: Component<CombinedDialogProps> = (props) => {
  const [{ t }] = useTranslation();

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
    'buttonOkVariant',
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
          <Button variant={local.buttonOkVariant} onClick={local.onOk}>
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
