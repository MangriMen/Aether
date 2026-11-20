import { useMutation, useQuery } from '@tanstack/solid-query';

import { showError } from '@/features/errors';
import { updateCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

import { updateKeys } from './updateQueryKeys';

/** The saved record of a game that has just started may come a moment after it. */
const GAMES_CARRY_ON_REFETCH_MS = 5000;

export const useCheckUpdate = () =>
  useQuery(() => ({
    queryKey: updateKeys.check(),
    queryFn: updateCommands.checkForUpdates,
  }));

/**
 * Whether the games that run would carry on across the restart of an update. Asked only while
 * a game runs.
 */
export const useGamesCarryOnUpdate = (games: () => readonly string[]) =>
  useQuery(() => ({
    queryKey: updateKeys.gamesCarryOn(games()),
    queryFn: updateCommands.gamesCarryOnUpdate,
    enabled: games().length > 0,
    staleTime: 0,
    refetchInterval: GAMES_CARRY_ON_REFETCH_MS,
  }));

export const useInstallUpdate = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    /** With `allowRunningGames` the games that run are left running across the restart. */
    mutationFn: (allowRunningGames: boolean) =>
      updateCommands.installUpdate(allowRunningGames),
    // `showError` logs what the backend did not (an error that is not a launcher one).
    onError: (err) => {
      showError({ title: t('update.updateError'), err, t });
    },
  }));
};
