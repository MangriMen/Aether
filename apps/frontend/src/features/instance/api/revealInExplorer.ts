import { useMutation, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/features/errors';
import { applicationCommands, instanceCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

import { INSTANCE_QUERY_KEYS } from './instanceQueryKeys';

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

/** Reveals the folder of an instance; the path is asked for on the click, not for every card. */
export const useRevealInstanceDir = () => {
  const queryClient = useQueryClient();
  const reveal = useRevealInExplorer();

  return useMutation(() => ({
    mutationFn: async (id: string) => {
      const path = await queryClient.fetchQuery({
        queryKey: [...INSTANCE_QUERY_KEYS.DIR(id)],
        queryFn: () => instanceCommands.getDir(id),
        staleTime: Infinity,
      });

      await reveal.mutateAsync({ path });
    },
  }));
};
