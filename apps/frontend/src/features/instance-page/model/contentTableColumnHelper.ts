import { createColumnHelper } from '@tanstack/solid-table';

import type { ContentFile } from '@/features/content';
import type { AppTableFeatures } from '@/shared/ui-kit';

export const contentTableColumnHelper = createColumnHelper<
  AppTableFeatures,
  ContentFile
>();
