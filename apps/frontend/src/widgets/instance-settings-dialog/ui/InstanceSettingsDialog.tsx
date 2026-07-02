import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

import { useNavigate } from '@solidjs/router';
import { useQueryClient } from '@tanstack/solid-query';
import { createMemo, createSignal, onMount, Show, splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { useInstance } from '@/entities/instances';
import { settingsCache } from '@/entities/settings';
import { useTranslation } from '@/shared/model';
import { Dialog, DialogContent } from '@/shared/ui';

import InstanceSettingsDialogBody from './InstanceSettingsDialogBody';
import InstanceSettingsDialogHeader from './InstanceSettingsDialogHeader';

export type InstanceSettingsDialogProps = DialogRootProps & {
  instanceId?: Instance['id'];
};

export const InstanceSettingsDialog: Component<InstanceSettingsDialogProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['instanceId', 'onOpenChange']);

  const [{ t }] = useTranslation();

  const [isOpen, setIsOpen] = createSignal(true);

  const navigate = useNavigate();

  const instanceId = createMemo(() =>
    local.instanceId ? decodeURIComponent(local.instanceId) : undefined,
  );

  const queryClient = useQueryClient();

  const instance = useInstance(() => instanceId());

  const onOpenChange = (open: boolean) => {
    if (local.onOpenChange) {
      local.onOpenChange(open);
    } else {
      setIsOpen(false);
      setTimeout(() => {
        navigate(-1);
      }, 200);
    }
  };

  onMount(() => {
    settingsCache.prefetch.maxRam(queryClient);
  });

  return (
    <Dialog open={isOpen()} onOpenChange={onOpenChange} {...others}>
      <DialogContent
        class='
        w-225 pb-0 pl-0 flex max-w-[calc(100%-80px)] flex-col
      '
      >
        <Show
          when={instance.data}
          fallback={<span>{t('instance.instanceNotFound')}</span>}
        >
          {(instance) => (
            <>
              <InstanceSettingsDialogHeader instance={instance()} />
              <InstanceSettingsDialogBody class='pl-6' instance={instance()} />
            </>
          )}
        </Show>
      </DialogContent>
    </Dialog>
  );
};
