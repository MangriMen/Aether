import type { ContentType } from '@/features/content';
import type { ModLoader } from '@/features/minecraft';

import { isModLoader } from '@/features/minecraft';

/**
 * The loader an install is pinned to: only mods depend on one, the instance's own loader wins
 * over a filter, and a filter that is not a loader name (it comes from the URL) is ignored.
 */
export const getInstallLoader = (
  contentType: ContentType,
  instanceLoader: ModLoader | undefined,
  filterLoader: string | undefined,
): ModLoader | undefined => {
  if (contentType !== 'mod') {
    return undefined;
  }

  return (
    instanceLoader ?? (isModLoader(filterLoader) ? filterLoader : undefined)
  );
};
