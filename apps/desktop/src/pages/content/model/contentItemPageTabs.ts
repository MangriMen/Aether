import type { ComponentProps } from 'solid-js';

import type { ContentItem } from '../../../entities/instances';
import type { TabConfig } from '../../../shared/model';

import { ContentDescriptionTab } from '../ui/ContentItemPageTabs/ContentDescriptionTab';
import { ContentVersionsTab } from '../ui/ContentItemPageTabs/ContentVersionsTab';

export const ContentItemPageTab = {
  Description: 'description',
  Versions: 'versions',
} as const;

export type ContentItemPageTab =
  (typeof ContentItemPageTab)[keyof typeof ContentItemPageTab];

export interface ContentItemPageTabProps {
  item: ContentItem | undefined;
}

export const CONTENT_ITEM_PAGE_TABS_DEFINITION = [
  {
    value: ContentItemPageTab.Description,
    label: 'description',
    component: ContentDescriptionTab,
  },
  {
    value: ContentItemPageTab.Versions,
    label: 'versions',
    component: ContentVersionsTab,
  },
] as const satisfies TabConfig<
  ContentItemPageTab,
  ComponentProps<'div'> & ContentItemPageTabProps
>[];

export const ContentItemPageTabs = new Set<string>(
  Object.values(ContentItemPageTab),
);

export const isContentItemPageTab = (
  value: string,
): value is ContentItemPageTab => ContentItemPageTabs.has(value);
