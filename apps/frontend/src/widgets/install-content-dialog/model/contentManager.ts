import type { Accessor } from 'solid-js';

import type { ContentItem, ProviderId } from '@/entities/instances';
import type { Instance } from '@/features/instance';

export interface ContentManager {
  // Getters
  providerId: Accessor<ProviderId | undefined>;
  // Actions
  installContent: (
    item: ContentItem,
    instanceId?: Instance['id'],
  ) => Promise<void>;
  createIsInstalling: (
    contentId: Accessor<ContentItem['id'] | undefined>,
    instanceId: Accessor<Instance['id'] | undefined>,
  ) => Accessor<boolean>;
  createIsInstalled: (
    contentId: Accessor<ContentItem['id'] | undefined>,
    instanceId: Accessor<Instance['id'] | undefined>,
  ) => Accessor<boolean>;
}
