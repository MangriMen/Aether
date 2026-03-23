import {
  CONTENT_TYPES,
  ContentType,
  type Instance,
} from '@/entities/instances';
import { ModLoader } from '@/entities/minecraft';

export const getAvailableContentTypes = (
  instance?: Instance,
  isInstanceContentPage?: boolean,
): readonly ContentType[] | undefined => {
  if (!isInstanceContentPage) {
    // Undefined means all content types
    return;
  }

  if (!instance) {
    return [];
  }

  if (instance.loader === ModLoader.Vanilla) {
    return [ContentType.ResourcePack, ContentType.DataPack];
  }

  return CONTENT_TYPES;
};
