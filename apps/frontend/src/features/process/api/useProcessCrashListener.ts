import { useTranslation } from '@/shared/i18n';
import { showToast } from '@/shared/ui-kit';

import type { ProcessEvent } from '../model/processEvent';

import { useProcessEventListener } from './useProcessEventListener';

export const useProcessCrashListener = () => {
  const [{ t }] = useTranslation();

  useProcessEventListener((event: ProcessEvent) => {
    if (event.event !== 'finished' || !event.crashed) {
      return;
    }

    showToast({
      title:
        event.exitCode === null
          ? t('events.gameCrashed')
          : t('events.gameCrashedCode', { code: event.exitCode }),
      variant: 'destructive',
    });
  });
};
