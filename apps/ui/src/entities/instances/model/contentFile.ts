import type { ContentFileDto } from '../api';

import { providerIdToString, type ProviderId } from './contentItem';

export type ContentFile = ContentFileDto;

export const getContentIdFromUpdateInfo = (
  contentFile: ContentFile,
  providerId: ProviderId,
): string | undefined =>
  contentFile.update?.[providerIdToString(providerId)].contentId;
