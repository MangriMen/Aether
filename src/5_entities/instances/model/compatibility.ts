import type { ContentItem } from './contentItem';

export interface ContentCompatibilityCheckParams {
  provider: string;
  contentItem: ContentItem;
}

export interface ContentCompatibilityResult {
  isCompatible: boolean;
}
