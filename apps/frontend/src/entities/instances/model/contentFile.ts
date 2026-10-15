import type { ContentFileDto } from '../api';

import { providerIdToString, type ProviderId } from './contentItem';

export type ContentFile = ContentFileDto;

export const getContentIdFromUpdateInfo = (
  contentFile: ContentFile,
  providerId: ProviderId,
): string | undefined =>
  contentFile.update?.[providerIdToString(providerId)]?.contentId;

/** Project name from the provider, then the file's own name, if any. */
export const getContentFileTitle = (
  contentFile: ContentFile,
): string | undefined =>
  contentFile.metadata?.name ?? contentFile.name ?? undefined;

/** Cached icon first, then the remote one. */
export const getContentFileIconSrc = (
  contentFile: ContentFile,
): string | undefined =>
  contentFile.metadata?.iconPath ?? contentFile.metadata?.iconUrl ?? undefined;
