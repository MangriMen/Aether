import { getTranslatedError } from '@/features/errors';
import { useTranslation } from '@/shared/i18n';
import { showToast } from '@/shared/ui-kit';

import { useProcessEventListener } from './useProcessEventListener';

/** The post-exit command could not run: the message names the program and the OS reason only. */
export const usePostExitHookFailedListener = () => {
  const [{ t }] = useTranslation();

  useProcessEventListener((event) => {
    if (event.event !== 'post_exit_hook_failed' || !event.hookError) {
      return;
    }

    showToast({
      title: t('events.postExitHookFailed'),
      description: getTranslatedError(
        { type: 'hook', payload: event.hookError },
        t,
      ),
      variant: 'destructive',
    });
  });
};
