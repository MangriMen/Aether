import type { ContentType } from './contentType';

import {
  providerIdToString,
  type ProviderId,
  type ProviderIdString,
} from './contentItem';

export interface ContentFile {
  contentPath: string;
  contentType: ContentType;
  disabled: boolean;
  fileName: string;
  hash: string;
  name?: string;
  size: number;
  updateProviderId?: ProviderId;
  update: Record<ProviderIdString, { contentId: string }>;
}

export const getContentIdFromUpdateInfo = (
  contentFile: ContentFile,
  providerId: ProviderId,
) => {
  return contentFile.update?.[providerIdToString(providerId)].contentId;
};
