import { ROUTES } from '@/shared/config';
import { searchParamsToQueryString } from '@/shared/lib';

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

/** The file has neither cached project metadata nor an update provider. */
export const isLocalContentFile = (contentFile: ContentFile): boolean =>
  !contentFile.metadata && !contentFile.updateProviderId;

/** First line of the content cell: project title, or the file name for local files. */
export const getContentFileDisplayTitle = (contentFile: ContentFile): string =>
  isLocalContentFile(contentFile)
    ? contentFile.fileName
    : (getContentFileTitle(contentFile) ?? contentFile.fileName);

export const getContentFileAuthor = (
  contentFile: ContentFile,
): string | undefined => contentFile.metadata?.author ?? undefined;

export const getContentFileVersionNumber = (
  contentFile: ContentFile,
): string | undefined =>
  contentFile.versionMetadata?.versionNumber ?? undefined;

/** Version number is known, is being fetched now, or is not to come. */
export type ContentFileVersionState = 'known' | 'pending' | 'unknown';

export const getContentFileVersionState = (
  contentFile: ContentFile,
): ContentFileVersionState => {
  if (getContentFileVersionNumber(contentFile)) {
    return 'known';
  }

  return contentFile.versionMetadataPending ? 'pending' : 'unknown';
};

export const getContentFileVersionSortValue = (
  contentFile: ContentFile,
): string => getContentFileVersionNumber(contentFile) ?? contentFile.fileName;

/**
 * Link to the project page of the file. The page has no way to preselect the
 * versions tab, so this opens the project itself.
 */
export const getContentFileProjectHref = (
  contentFile: ContentFile,
  instanceId?: string,
): string | undefined => {
  const providerId = contentFile.updateProviderId;

  if (!providerId) {
    return undefined;
  }

  const contentId =
    getContentIdFromUpdateInfo(contentFile, providerId) ??
    contentFile.metadata?.contentId;

  if (!contentId) {
    return undefined;
  }

  const query = searchParamsToQueryString({
    instance: instanceId,
    plugin: providerId.pluginId,
    capability: providerId.capabilityId,
  });

  const base = ROUTES.CONTENT_ITEM(contentId);
  return query ? `${base}?${query}` : base;
};
