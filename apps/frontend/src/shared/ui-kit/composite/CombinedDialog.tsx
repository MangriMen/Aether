import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps, JSX } from 'solid-js';

import { mergeProps, Show, splitProps } from 'solid-js';

import type { ButtonProps } from '../primitives/Button';
import type { DialogContentProps } from '../primitives/Dialog';

import { Button } from '../primitives/Button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
} from '../primitives/Dialog';
import { useUiKitLabels } from '../UiKitLabels';

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
  const labels = useUiKitLabels();

  const mergedProps = mergeProps(
    {
      buttonOkText: labels.ok || 'Ok',
      buttonCancelText: labels.cancel || 'Cancel',
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
      <DialogContent variant={local.variant}>
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
