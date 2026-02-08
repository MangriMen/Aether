import {
  ATOMIC_CONTENT_TYPES,
  ContentType,
  type Instance,
} from '@/entities/instances';
import { ModLoader } from '@/entities/minecraft';

export const getAvailableContentTypes = (
  instance?: Instance,
  hasInstanceId?: boolean,
): ContentType[] | undefined => {
  if (!hasInstanceId) {
    // Undefined means all content types
    return;
  }

  if (!instance) {
    return [];
  } else if (instance.loader == ModLoader.Vanilla) {
    return [ContentType.ResourcePack, ContentType.DataPack];
  }

  return ATOMIC_CONTENT_TYPES;
};
