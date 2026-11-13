import { useMutation, useQuery } from '@tanstack/solid-query';

import { showError } from '@/features/errors';
import { updateCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

import { updateKeys } from './updateQueryKeys';

export const useCheckUpdate = () =>
  useQuery(() => ({
    queryKey: updateKeys.check(),
    queryFn: updateCommands.checkForUpdates,
  }));

export const useInstallUpdate = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: updateCommands.installUpdate,
    // `showError` logs what the backend did not (an error that is not a launcher one).
    onError: (err) => {
      showError({ title: t('update.updateError'), err, t });
    },
  }));
};
