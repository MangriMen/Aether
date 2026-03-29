import { ATOMIC_CONTENT_TYPES, ContentType } from '@/entities/instances';
import { ModLoader } from '@/entities/minecraft';

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

  if (instanceLoader === ModLoader.Vanilla) {
    return [ContentType.ResourcePack, ContentType.DataPack];
  }

  return ATOMIC_CONTENT_TYPES;
};
