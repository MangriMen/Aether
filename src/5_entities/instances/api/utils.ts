import { useMutation } from '@tanstack/solid-query';
import { revealInExplorerRaw } from './rawApi';
import { useTranslation } from '@/6_shared/model';
import { showError } from '@/6_shared/lib/showError';

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
