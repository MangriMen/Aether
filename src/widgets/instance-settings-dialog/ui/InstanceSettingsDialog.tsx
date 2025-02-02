import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';
import { createMemo, Show, splitProps } from 'solid-js';

import { Dialog, DialogContent } from '@/shared/ui';

import { useInstance } from '@/entities/instances';

import InstanceSettingsDialogBody from './InstanceSettingsDialogBody';
import InstanceSettingsDialogHeader from './InstanceSettingsDialogHeader';
import { useNavigate } from '@solidjs/router';

export type InstanceSettingsDialogProps = DialogRootProps & {
  id: string;
};

export const InstanceSettingsDialog: Component<InstanceSettingsDialogProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['id', 'onOpenChange']);

  const navigate = useNavigate();

  const id = createMemo(() => decodeURIComponent(local.id));

  // eslint-disable-next-line solid/reactivity
  const instance = useInstance(id);

  const onOpenChange = (open: boolean) => {
    if (local.onOpenChange) {
      local.onOpenChange(open);
    } else {
      navigate(-1);
    }
  };

  return (
    <Dialog defaultOpen onOpenChange={onOpenChange} {...others}>
      <DialogContent class='w-[900px] max-w-full bg-secondary-dark'>
        <Show when={instance()} fallback={<span>Instance not found</span>}>
          {(instance) => (
            <>
              <InstanceSettingsDialogHeader instance={instance()} />
              <InstanceSettingsDialogBody instance={instance()} />
            </>
          )}
        </Show>
      </DialogContent>
    </Dialog>
  );
};
