import { useMutation } from '@tanstack/solid-query';

import { isCancelledError, showError } from '@/features/errors';
import { instanceCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

export const useLaunchInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (id: string) => instanceCommands.launch(id),
    onError: (err, id) => {
      if (isCancelledError(err)) {
        return;
      }

      showError({
        title: t('instance.launchError', { id }),
        err,
        t,
      });
    },
  }));
};

/** Stops the instance: calls its launch off while it has no game yet, or ends its game. */
export const useStopInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (id: string) => instanceCommands.stop(id),
    onError: (err) => {
      showError({
        title: t('instance.stopError'),
        err,
        t,
      });
    },
  }));
};
