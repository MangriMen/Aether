import type { PolymorphicProps } from '@kobalte/core';

import { type ValidComponent } from 'solid-js';

import type { CombinedDialogProps } from '@/shared/ui-kit';

import { useTranslation } from '@/shared/i18n';
import { CombinedDialog } from '@/shared/ui-kit';

export type UpdateAndRestartWarningDialogProps = Omit<
  CombinedDialogProps,
  'header' | 'description'
>;

export const UpdateAndRestartWarningDialog = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, UpdateAndRestartWarningDialogProps>,
) => {
  const [{ t }] = useTranslation();

  return (
    <CombinedDialog
      header={t('update.restartWarningTitle')}
      description={t('update.restartWarningDescription')}
      {...props}
    />
  );
};
