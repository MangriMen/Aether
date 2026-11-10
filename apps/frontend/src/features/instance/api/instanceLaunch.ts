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

export const useStopInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (uuid: string) => instanceCommands.stop(uuid),
    onError: (err) => {
      showError({
        title: t('instance.stopError'),
        err,
        t,
      });
    },
  }));
};
