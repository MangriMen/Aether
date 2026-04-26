import type { Accessor } from 'solid-js';

import type { ContentItem, Instance, ProviderId } from '@/entities/instances';

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
