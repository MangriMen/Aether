import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ParentComponent } from 'solid-js';

import { useNavigate } from '@solidjs/router';
import { useQueryClient } from '@tanstack/solid-query';
import {
  createMemo,
  createSignal,
  Match,
  onMount,
  splitProps,
  Switch,
} from 'solid-js';

import type { Instance } from '@/features/instance';

import { useInstance } from '@/features/instance';
import { settingsCache } from '@/features/settings';
import { useTranslation } from '@/shared/i18n';
import { Dialog, DialogContent, LoadError } from '@/shared/ui-kit';

import InstanceSettingsDialogBody from './InstanceSettingsDialogBody';
import InstanceSettingsDialogHeader from './InstanceSettingsDialogHeader';

// The settings body is h-98 under a 32px header and 8px gap, and the dialog content has no left
// padding, so a state shown instead of the settings takes the same total height (h-108) and brings
// the padding.
const StatePlaceholder: ParentComponent = (props) => (
  <div class='h-108 pl-6 flex grow items-center justify-center'>
    {props.children}
  </div>
);

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
        <Switch
          fallback={
            <StatePlaceholder>
              <span>{t('instance.instanceNotFound')}</span>
            </StatePlaceholder>
          }
        >
          <Match when={instance.data}>
            {(instance) => (
              <>
                <InstanceSettingsDialogHeader instance={instance()} />
                <InstanceSettingsDialogBody
                  class='pl-6'
                  instance={instance()}
                />
              </>
            )}
          </Match>
          <Match when={instance.isLoading || instance.isFetching}>
            <StatePlaceholder>
              <span>{t('common.loading')}</span>
            </StatePlaceholder>
          </Match>
          <Match when={instance.isError && !instance.isNotFound}>
            <StatePlaceholder>
              <LoadError
                message={t('common.loadFailed')}
                retryLabel={t('common.retry')}
                onRetry={instance.refetch}
              />
            </StatePlaceholder>
          </Match>
        </Switch>
      </DialogContent>
    </Dialog>
  );
};
