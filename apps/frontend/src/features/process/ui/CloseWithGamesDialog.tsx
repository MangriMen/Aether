import { useTranslation } from '@/shared/i18n';
import { CombinedDialog } from '@/shared/ui-kit';

import { useCloseConfirmationRequest } from '../api/useCloseConfirmationRequest';

/** Asks before the launcher is closed under a running game. */
export const CloseWithGamesDialog = () => {
  const [{ t }] = useTranslation();
  const request = useCloseConfirmationRequest();

  return (
    <CombinedDialog
      open={request.isRequested()}
      onOpenChange={(open) => {
        if (!open) {
          request.cancel();
        }
      }}
      header={t('events.closeWithGamesTitle')}
      description={t('events.closeWithGamesDescription')}
      buttonOkText={t('events.closeWithGamesConfirm')}
      buttonOkVariant='destructive'
      onOk={request.confirm}
      onCancel={request.cancel}
    />
  );
};
