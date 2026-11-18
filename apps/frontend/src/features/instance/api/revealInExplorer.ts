import { useMutation } from '@tanstack/solid-query';

import { showError } from '@/features/errors';
import { applicationCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

export const useRevealInExplorer = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ path, exact }: { path: string; exact?: boolean }) =>
      applicationCommands.revealInExplorer(path, exact ?? true),
    onError: (err) => {
      showError({
        title: t('common.revealInExplorerError'),
        err,
        t,
      });
    },
  }));
};
