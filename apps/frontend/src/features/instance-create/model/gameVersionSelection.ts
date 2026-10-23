type WithId = { id: string };

export type GameVersionSelection = {
  gameVersion: string | undefined;
  // The loader version belongs to the game version; it is dropped whenever that changes.
  loaderVersion: string | undefined;
};

/**
 * Keeps the selected game version when it is still offered, otherwise resets it
 * (and the dependent loader version) so the form never holds a value the select does not show.
 */
export const reconcileGameVersion = (
  selected: string | undefined,
  loaderVersion: string | undefined,
  available: readonly WithId[],
): GameVersionSelection => {
  if (selected !== undefined && available.some((v) => v.id === selected)) {
    return { gameVersion: selected, loaderVersion };
  }

  return { gameVersion: undefined, loaderVersion: undefined };
};
