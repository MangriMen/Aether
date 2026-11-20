export const updateKeys = {
  all: ['update'] as const,
  check: () => [...updateKeys.all, 'check'] as const,
  gamesCarryOn: (games: readonly string[]) =>
    [...updateKeys.all, 'gamesCarryOn', ...games] as const,
};
