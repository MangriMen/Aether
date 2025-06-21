import { useMutation } from '@tanstack/solid-query';
import { revealInExplorerRaw } from './rawApi';
import { showToast } from '@/shared/ui';
import {
  getTranslatedError,
  isLauncherError,
  useTranslation,
} from '@/6_shared/model';

export const useRevealInExplorer = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ path, exact }: { path: string; exact?: boolean }) =>
      revealInExplorerRaw(path, exact ?? true),
    onError: (err) => {
      if (isLauncherError(err)) {
        showToast({
          title: t('common.revealInExplorerError'),
          description: getTranslatedError(err, t),
          variant: 'destructive',
        });
      }
    },
  }));
};
