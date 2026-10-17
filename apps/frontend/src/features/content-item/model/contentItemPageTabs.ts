import type { ComponentProps } from 'solid-js';

import type { ContentItem } from '@/features/content';
import type { TabConfig } from '@/shared/ui-kit';

import { ContentDescriptionTab } from '../ui/ContentDescriptionTab';
import { ContentVersionsTab } from '../ui/ContentVersionsTab';

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
