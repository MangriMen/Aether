import { useMutation } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { revealInExplorerRaw } from '../../api';

export const useRevealInExplorer = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ path, exact }: { path: string; exact?: boolean }) =>
      revealInExplorerRaw(path, exact ?? true),
    onError: (err) => {
      showError({
        title: t('common.revealInExplorerError'),
        err,
        t,
      });
    },
  }));
};
