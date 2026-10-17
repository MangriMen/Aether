import type { Instance } from '@/features/instance';

import type { ContentGetParams } from '../contentItem';

export const instanceToContentGetParams = (
  instance: Instance,
): ContentGetParams | undefined => {
  const packInfo = instance.packInfo;

  if (!packInfo) {
    return;
  }

  return {
    contentId: packInfo.modpackId,
    providerId: packInfo.providerId,
  };
};
