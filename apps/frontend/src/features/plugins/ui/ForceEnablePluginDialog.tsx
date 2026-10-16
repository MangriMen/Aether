import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui';

export type ForceEnablePluginDialogProps = DialogRootProps & {
  pluginName: string;
  onConfirm: () => void;
};

export const ForceEnablePluginDialog: Component<
  ForceEnablePluginDialogProps
> = (props) => {
  const [local, others] = splitProps(props, ['pluginName', 'onConfirm']);

  const [{ t }] = useTranslation();

  const handleConfirm = () => {
    local.onConfirm();
  };

  return (
    <Dialog {...others}>
      <DialogContent variant='warning'>
        <DialogHeader>
          <DialogTitle>{t('plugin.forceEnableTitle')}</DialogTitle>
          <DialogDescription>
            {t('plugin.forceEnableDescription')}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant='outline' onClick={() => props.onOpenChange?.(false)}>
            {t('plugin.forceEnableCancel')}
          </Button>
          <Button variant='warning' onClick={handleConfirm}>
            {t('plugin.forceEnableConfirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
