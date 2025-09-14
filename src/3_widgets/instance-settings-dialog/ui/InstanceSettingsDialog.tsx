import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';
import { createMemo, onMount, Show, splitProps } from 'solid-js';

import { Dialog, DialogContent } from '@/shared/ui';

import type { Instance } from '@/entities/instances';
import { useInstance } from '@/entities/instances';

import InstanceSettingsDialogBody from './InstanceSettingsDialogBody';
import InstanceSettingsDialogHeader from './InstanceSettingsDialogHeader';
import { useNavigate } from '@solidjs/router';
import { useQueryClient } from '@tanstack/solid-query';
import { prefetchMaxRam } from '@/entities/settings';

export type InstanceSettingsDialogProps = DialogRootProps & {
  instanceId: Instance['id'];
};

export const InstanceSettingsDialog: Component<InstanceSettingsDialogProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['instanceId', 'onOpenChange']);

  const navigate = useNavigate();

  const instance_id = createMemo(() => decodeURIComponent(local.instanceId));

  const queryClient = useQueryClient();

  const instance = useInstance(() => instance_id());

  const onOpenChange = (open: boolean) => {
    if (local.onOpenChange) {
      local.onOpenChange(open);
    } else {
      navigate(-1);
    }
  };

  onMount(() => {
    prefetchMaxRam(queryClient);
  });

  return (
    <Dialog defaultOpen onOpenChange={onOpenChange} {...others}>
      <DialogContent class='flex w-[900px] max-w-[calc(100%-80px)] flex-col bg-secondary-dark'>
        <Show when={instance.data} fallback={<span>Instance not found</span>}>
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
