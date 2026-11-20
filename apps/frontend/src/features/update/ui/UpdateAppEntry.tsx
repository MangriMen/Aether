import type { Component } from 'solid-js';

import { createMemo, createSignal, Show } from 'solid-js';

import type { SettingsEntryProps } from '@/shared/settings-layout';

import { showError } from '@/features/errors';
import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';
import { Button } from '@/shared/ui-kit';

import { useCheckUpdate } from '../api/updateQueries';
import { useRunningGames } from '../lib/useRunningGames';
import { useUpdate } from '../lib/useUpdate';
import { checkIsUpdateAvailable } from '../model/checkIsUpdateAvailable';
import { RunningGamesHint } from './RunningGamesHint';
import { UpdateAndRestartWarningDialog } from './UpdateAndRestartWarningDialog';
import { UpdateDescription } from './UpdateDescription';
import { WhatsNew } from './WhatsNew';

export type UpdateAppEntryProps = SettingsEntryProps;

export const UpdateAppEntry: Component<UpdateAppEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  const handleCheckUpdates = async () => {
    const result = await update.refetch();

    if (result.isError) {
      showError({ title: t('update.fetchError'), err: result.error, t });
    }
  };

  const { isUpdating, updateAndRestart } = useUpdate();

  const runningGames = useRunningGames();

  const [isDialogOpen, setIsDialogOpen] = createSignal(false);

  const handleInstallUpdate = async () => {
    setIsDialogOpen(false);

    if (!update.data) {
      return;
    }

    await updateAndRestart(update.data, runningGames.keepsGames());
  };

  return (
    <div>
      <SettingsEntry
        title={t('settings.tab.update')}
        description={
          <UpdateDescription
            isUpdateAvailable={isUpdateAvailable()}
            isChecking={update.isPending}
            isFailed={update.isError}
            update={update.data}
          />
        }
        {...props}
      >
        <UpdateButton
          isUpdateAvailable={isUpdateAvailable()}
          isFetching={update.isFetching}
          isUpdating={isUpdating()}
          isBlocked={runningGames.isBlocked()}
          onCheckUpdates={handleCheckUpdates}
          onSubmitUpdate={() => setIsDialogOpen(true)}
        />
      </SettingsEntry>
      <Show when={isUpdateAvailable()}>
        <RunningGamesHint
          games={runningGames.isBlocked() ? runningGames.names() : []}
        />
      </Show>
      <UpdateAndRestartWarningDialog
        open={isDialogOpen()}
        keptGames={runningGames.keepsGames() ? runningGames.names() : []}
        onOpenChange={setIsDialogOpen}
        onOk={handleInstallUpdate}
        onCancel={() => setIsDialogOpen(false)}
      />
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
  /** A game runs: the launcher would restart under it. */
  isBlocked: boolean;
  onCheckUpdates: () => void;
  onSubmitUpdate: () => void;
}

const UpdateButton: Component<UpdateButtonProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Show
      when={props.isUpdateAvailable}
      fallback={
        <Button loading={props.isFetching} onClick={props.onCheckUpdates}>
          {t('settings.checkForUpdates')}
        </Button>
      }
    >
      <Button
        loading={props.isUpdating}
        disabled={props.isBlocked}
        onClick={props.onSubmitUpdate}
      >
        {t('settings.installAndRestart')}
      </Button>
    </Show>
  );
};
