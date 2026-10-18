import { createColumnHelper } from '@tanstack/solid-table';

import type { ContentVersion } from '@/features/content';
import type { AppTableFeatures } from '@/shared/ui-kit';

export const contentVersionsColumnHelper = createColumnHelper<
  AppTableFeatures,
  ContentVersion
>();
