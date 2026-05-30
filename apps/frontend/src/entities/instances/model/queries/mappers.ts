import type { ContentGetParams } from '../contentItem';
import type { Instance } from '../instance';

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
