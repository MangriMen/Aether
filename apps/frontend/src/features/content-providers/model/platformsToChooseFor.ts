import type { PlatformProviders } from './groupProvidersByPlatform';

/**
 * The platforms the settings offer a choice for: those served by several
 * providers, and any with a stored choice — even served by one provider or by
 * none, once the chosen plugin is gone. Resolving a ref to such a platform
 * fails until the choice is changed, so it must stay where it can be changed.
 */
export const platformsToChooseFor = (
  groups: PlatformProviders[],
  chosen: Partial<Record<string, string>> | undefined,
): PlatformProviders[] => {
  const isChosen = (platform: string) => Boolean(chosen?.[platform]);

  const served = groups.filter(
    (group) => group.providers.length > 1 || isChosen(group.platform),
  );

  const known = new Set(groups.map((group) => group.platform));
  const unserved = Object.keys(chosen ?? {})
    .filter((platform) => isChosen(platform) && !known.has(platform))
    .sort()
    .map((platform) => ({ platform, providers: [] }));

  return [...served, ...unserved];
};
