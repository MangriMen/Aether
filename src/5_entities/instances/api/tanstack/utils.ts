import { useMutation } from '@tanstack/solid-query';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import { revealInExplorerRaw } from '../rawApi';

export const useRevealInExplorer = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ exact, path }: { exact?: boolean; path: string }) =>
      revealInExplorerRaw(path, exact ?? true),
    onError: (err) => {
      showError({
        err,
        t,
        title: t('common.revealInExplorerError'),
      });
    },
  }));
};
