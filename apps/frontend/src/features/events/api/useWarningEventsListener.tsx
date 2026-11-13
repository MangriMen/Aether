import {
  ErrorDetails,
  getReasonDetails,
  getTranslatedWarning,
} from '@/features/errors';
import { useEventListener, eventsEvents } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';
import { logDebug } from '@/shared/log';
import { showToast } from '@/shared/ui-kit';

export const useWarningEventsListener = () => {
  const [{ t }] = useTranslation();

  useEventListener(eventsEvents.warningEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    const { warning } = e.payload;
    const details = getReasonDetails(warning);

    showToast({
      title: t('warnings.title'),
      description: getTranslatedWarning(warning, t),
      action: details
        ? () => <ErrorDetails details={details} t={t} />
        : undefined,
      variant: 'warning',
    });
  });
};
