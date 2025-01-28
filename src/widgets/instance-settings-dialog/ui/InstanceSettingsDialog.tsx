import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';

import { Dialog, DialogContent } from '@/shared/ui';

import type { Instance } from '@/entities/instance';

import InstanceSettingsDialogBody from './InstanceSettingsDialogBody';
import InstanceSettingsDialogHeader from './InstanceSettingsDialogHeader';

export type InstanceSettingsDialogProps = DialogRootProps & {
  instance: Instance;
};

const InstanceSettingsDialog: Component<InstanceSettingsDialogProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['instance']);

  return (
    <Dialog defaultOpen open={true} {...others}>
      <DialogContent class='w-[900px] max-w-full bg-secondary-dark'>
        <InstanceSettingsDialogHeader instance={local.instance} />
        <InstanceSettingsDialogBody instance={local.instance} />
      </DialogContent>
    </Dialog>
  );
};

export default InstanceSettingsDialog;
