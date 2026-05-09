import type { ModLoader } from '@/entities/minecraft';

import { ATOMIC_CONTENT_TYPES, ContentType } from '@/entities/instances';
import { getOrCreateFeatureFlagsStore } from '@/shared/model/featureFlags';

export const getAvailableContentTypes = (
  instanceLoader: ModLoader | undefined,
  isInstanceContentPage: boolean,
): readonly ContentType[] | undefined => {
  const [ff] = getOrCreateFeatureFlagsStore();

  if (!isInstanceContentPage) {
    if (!ff.isAllowInstallModpacks) {
      return [
        ContentType.Mod,
        ContentType.DataPack,
        ContentType.ResourcePack,
        ContentType.ShaderPack,
      ];
    }
    // Undefined means all content types
    return;
  }

  if (!instanceLoader) {
    return [];
  }

  if (instanceLoader === 'vanilla') {
    return [ContentType.ResourcePack, ContentType.DataPack];
  }

  return ATOMIC_CONTENT_TYPES;
};
