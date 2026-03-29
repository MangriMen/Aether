import type { ContentItem, ProviderId } from './contentItem';

export interface ContentCompatibilityCheckParams {
  providerId: ProviderId;
  contentItem: ContentItem;
}

export interface ContentCompatibilityResult {
  isCompatible: boolean;
}
