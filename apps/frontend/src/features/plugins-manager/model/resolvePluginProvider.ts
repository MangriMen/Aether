import type { PluginSourceType } from './pluginProvider';

/**
 * The provider to show: the user's pick if the backend still offers it, else the first one it
 * does; none while there is no list or it is empty.
 */
export const resolvePluginProvider = (
  providers: readonly PluginSourceType[],
  picked: PluginSourceType | null,
): PluginSourceType | null => {
  if (picked !== null && providers.includes(picked)) {
    return picked;
  }

  return providers[0] ?? null;
};
