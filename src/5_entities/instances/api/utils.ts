import { useMutation } from '@tanstack/solid-query';
import { revealInExplorerRaw } from './rawApi';
import { showToast } from '@/shared/ui';

export const useRevealInExplorer = () => {
  return useMutation(() => ({
    mutationFn: ({ path, exact }: { path: string; exact?: boolean }) =>
      revealInExplorerRaw(path, exact ?? true),
    onError: (error) => {
      showToast({
        title: 'Failed to open explorer',
        description: error.message,
        variant: 'destructive',
      });
    },
  }));
};
