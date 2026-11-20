import { onMount } from 'solid-js';

import {
  ErrorDetails,
  getReasonDetails,
  getTranslatedWarning,
} from '@/features/errors';
import {
  eventsEvents,
  processCommands,
  useReadyEventListener,
} from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';
import { logDebug, logError } from '@/shared/log';
import { showToast } from '@/shared/ui-kit';

type Warning = Parameters<typeof getTranslatedWarning>[0];

/** Warnings the user has to read and close: they say that something did not happen. */
const isPersistent = (warning: Warning) =>
  warning.code === 'POST_EXIT_HOOK_SKIPPED';

// The backend hands the held notices out once, so asking again is pointless. Kept per module,
// not per mount, so that a remount of the listener does not ask twice.
let noticesTaken = false;

export const useWarningEventsListener = () => {
  const [{ t }] = useTranslation();

  const show = (warning: Warning) => {
    const details = getReasonDetails(warning);

    showToast({
      title: t('warnings.title'),
      description: getTranslatedWarning(warning, t),
      action: details
        ? () => <ErrorDetails details={details} t={t} />
        : undefined,
      variant: 'warning',
      persistent: isPersistent(warning),
    });
  };

  const listening = useReadyEventListener(eventsEvents.warningEventDto, (e) => {
    logDebug('[EVENT][DEBUG]', e);

    show(e.payload.warning);
  });

  const takeNotices = async () => {
    if (noticesTaken) {
      return;
    }

    noticesTaken = true;

    try {
      // After the listener is registered: what comes later is sent as events.
      await listening;

      for (const warning of await processCommands.takeSkippedHookNotices()) {
        show(warning);
      }
    } catch (error) {
      logError('Failed to take the skipped post-exit hook notices:', error);
    }
  };

  onMount(() => {
    void takeNotices();
  });
};
