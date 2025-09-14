import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

import { useNavigate } from '@solidjs/router';
import { useQueryClient } from '@tanstack/solid-query';
import { createMemo, onMount, Show, splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { useInstance } from '@/entities/instances';
import { prefetchMaxRam } from '@/entities/settings';
import { Dialog, DialogContent } from '@/shared/ui';

import InstanceSettingsDialogBody from './InstanceSettingsDialogBody';
import InstanceSettingsDialogHeader from './InstanceSettingsDialogHeader';

export type InstanceSettingsDialogProps = {
  instanceId: Instance['id'];
} & DialogRootProps;

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
        <Show fallback={<span>Instance not found</span>} when={instance.data}>
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
