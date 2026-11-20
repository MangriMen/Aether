import type { PolymorphicProps } from '@kobalte/core';

import { type ValidComponent } from 'solid-js';

import type { CombinedDialogProps } from '@/shared/ui-kit';

import { useTranslation } from '@/shared/i18n';
import { CombinedDialog } from '@/shared/ui-kit';

export type UpdateAndRestartWarningDialogProps = Omit<
  CombinedDialogProps,
  'header' | 'description'
> & {
  /** The games that keep running across the restart. */
  keptGames?: string[];
};

export const UpdateAndRestartWarningDialog = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, UpdateAndRestartWarningDialogProps>,
) => {
  const [{ t }] = useTranslation();

  const description = () =>
    props.keptGames?.length
      ? `${t('update.restartWarningDescription')} ${t('update.gamesKeepRunning', { games: props.keptGames.join(', ') })}`
      : t('update.restartWarningDescription');

  return (
    <CombinedDialog
      header={t('update.restartWarningTitle')}
      description={description()}
      {...props}
    />
  );
};
