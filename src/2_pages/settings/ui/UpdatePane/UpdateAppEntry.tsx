import type { Component, ComponentProps } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import { useCheckUpdate, useInstallUpdate } from '@/entities/updates';
import { checkIsUpdateAvailable } from '@/entities/updates/model';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, SettingsEntry, showToast } from '@/shared/ui';

import { UpdateDescription } from './UpdateDescription';
import { WhatsNew } from './WhatsNew';

export type UpdateAppEntryProps = ComponentProps<'div'>;

export const UpdateAppEntry: Component<UpdateAppEntryProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  const handleCheckUpdates = async () => {
    const result = await update.refetch();

    if (result.isError) {
      showToast({
        title: t('update.fetchError'),
        variant: 'destructive',
      });
    }
  };

  const { isUpdating, updateAndRestart } = useInstallUpdate();

  const handleInstallUpdate = async () => {
    if (!update.data) {
      return;
    }

    await updateAndRestart(update.data);
  };

  return (
    <div>
      <SettingsEntry
        class={cn('items-start', local.class)}
        title={t('settings.checkForUpdates')}
        description={
          <UpdateDescription
            isUpdateAvailable={isUpdateAvailable()}
            update={update.data}
          />
        }
        {...others}
      >
        <div class='mt-6 flex h-full min-w-max flex-col justify-start gap-2'>
          <UpdateButton
            isUpdateAvailable={isUpdateAvailable()}
            isFetching={update.isFetching}
            isUpdating={isUpdating()}
            onCheckUpdates={handleCheckUpdates}
            onSubmitUpdate={handleInstallUpdate}
          />
        </div>
      </SettingsEntry>
      <Show when={update.data?.body}>
        {(body) => <WhatsNew changelogBody={body()} />}
      </Show>
    </div>
  );
};

interface UpdateButtonProps {
  isUpdateAvailable: boolean;
  isFetching: boolean;
  isUpdating: boolean;
  onCheckUpdates: () => void;
  onSubmitUpdate: () => void;
}

const UpdateButton: Component<UpdateButtonProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Show
      when={props.isUpdateAvailable}
      fallback={
        <Button
          variant='default'
          loading={props.isFetching}
          onClick={props.onCheckUpdates}
        >
          {t('settings.checkForUpdates')}
        </Button>
      }
    >
      <Button
        variant='default'
        loading={props.isUpdating}
        onClick={props.onSubmitUpdate}
      >
        {t('settings.installAndRestart')}
      </Button>
    </Show>
  );
};
