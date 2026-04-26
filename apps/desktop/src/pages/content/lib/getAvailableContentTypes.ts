import type { ModLoader } from '../../../entities/minecraft';

import { ATOMIC_CONTENT_TYPES, ContentType } from '../../../entities/instances';

export const getAvailableContentTypes = (
  instanceLoader: ModLoader | undefined,
  isInstanceContentPage: boolean,
): readonly ContentType[] | undefined => {
  if (!isInstanceContentPage) {
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
