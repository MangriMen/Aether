import { useMutation } from '@tanstack/solid-query';

import { isCancelledError, showError } from '@/features/errors';
import { instanceCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

export const useInstallInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ id, force }: { id: string; force?: boolean }) =>
      instanceCommands.install(id, force ?? false),
    onError: (err, { id }) => {
      if (isCancelledError(err)) {
        return;
      }

      showError({
        title: t('instance.installError', { id }),
        err,
        t,
      });
    },
  }));
};

export const useUpdateInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (id: string) => instanceCommands.update(id),
    onError: (err, id) => {
      if (isCancelledError(err)) {
        return;
      }

      showError({
        title: t('instance.updateError', { id }),
        err,
        t,
      });
    },
  }));
};
